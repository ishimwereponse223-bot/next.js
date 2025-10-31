use swc_common::{SourceMap, Span};
use swc_ecma_lexer::{Lexer, StringInput, Syntax, TsSyntax, token::Token};

/// A style marker at a specific byte offset in the source
/// Represents either the start or end of a styled region
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StyleMarker {
    /// Byte offset in the source (0-indexed)
    pub offset: usize,
    /// Whether this is a start (true) or end (false) marker
    pub is_start: bool,
    /// The token type being styled
    pub token_type: TokenType,
}

/// Highlighting information for a single line
/// Contains sorted style markers that should be applied when rendering the line
#[derive(Debug, Clone)]
pub struct LineHighlight {
    /// Line number (1-indexed)
    pub line: usize,
    /// Byte offset where this line starts in the source
    pub line_start_offset: usize,
    /// Byte offset where this line ends (exclusive) in the source
    pub line_end_offset: usize,
    /// Style markers for this line, sorted by offset
    /// Offsets are relative to line_start_offset
    pub markers: Vec<StyleMarker>,
}

/// Token types for syntax highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenType {
    Keyword,
    Identifier,
    String,
    Number,
    Regex,
    JsxTag,
    Punctuation,
    Comment,
}

/// ANSI color codes for token types
#[derive(Debug, Clone, Copy)]
pub struct ColorScheme {
    pub reset: &'static str,
    pub keyword: &'static str,
    pub identifier: &'static str,
    pub string: &'static str,
    pub number: &'static str,
    pub regex: &'static str,
    pub jsx_tag: &'static str,
    pub punctuation: &'static str,
    pub comment: &'static str,
    pub gutter: &'static str,
    pub marker: &'static str,
}

impl ColorScheme {
    /// Get a color scheme with ANSI colors (matching babel-code-frame)
    pub const fn colored() -> Self {
        Self {
            reset: "\x1b[0m",
            keyword: "\x1b[36m",       // cyan
            identifier: "\x1b[33m",    // yellow (for capitalized/jsx identifiers)
            string: "\x1b[32m",        // green
            number: "\x1b[35m",        // magenta
            regex: "\x1b[35m",         // magenta
            jsx_tag: "\x1b[33m",       // yellow
            punctuation: "\x1b[33m",   // yellow
            comment: "\x1b[90m",       // gray
            gutter: "\x1b[90m",        // gray
            marker: "\x1b[31m\x1b[1m", // red + bold
        }
    }

    /// Get a plain color scheme with no ANSI codes (all empty strings)
    pub const fn plain() -> Self {
        Self {
            reset: "",
            keyword: "",
            identifier: "",
            string: "",
            number: "",
            regex: "",
            jsx_tag: "",
            punctuation: "",
            comment: "",
            gutter: "",
            marker: "",
        }
    }

    /// Get the color for a token type
    pub fn color_for_token(&self, token_type: TokenType) -> &'static str {
        match token_type {
            TokenType::Keyword => self.keyword,
            TokenType::Identifier => self.identifier,
            TokenType::String => self.string,
            TokenType::Number => self.number,
            TokenType::Regex => self.regex,
            TokenType::JsxTag => self.jsx_tag,
            TokenType::Punctuation => self.punctuation,
            TokenType::Comment => self.comment,
        }
    }
}

/// Lex the entire source file and extract token information
/// Returns highlighting information for each line
pub fn extract_highlights(source: &str) -> Vec<LineHighlight> {
    // Build line offset map first
    let line_offsets = build_line_offset_map(source);

    // Create a SourceMap and SourceFile for the lexer
    let cm = SourceMap::default();
    let fm = cm.new_source_file(
        swc_common::FileName::Custom("input.tsx".into()).into(),
        source.to_string(),
    );

    // Configure syntax for TypeScript + JSX
    let syntax = Syntax::Typescript(TsSyntax {
        tsx: true,
        decorators: true,
        ..Default::default()
    });

    // Create lexer
    let input = StringInput::from(&*fm);
    let mut lexer = Lexer::new(syntax, Default::default(), input, None);

    // Lex all tokens and extract style markers
    let mut markers = Vec::new();

    loop {
        match lexer.next() {
            Some(token) => {
                if token.token == Token::Eof {
                    break;
                }

                // Classify token and add markers
                if let Some(token_type) = classify_token(&token.token) {
                    add_token_markers(&mut markers, token.span, token_type);
                }
            }
            None => {
                // End of input
                break;
            }
        }
    }

    // Sort markers by offset
    markers.sort();

    // Split markers into per-line highlights
    split_markers_by_line(&markers, &line_offsets)
}

/// Classify a token into a highlighting type
fn classify_token(token: &Token) -> Option<TokenType> {
    match token {
        // Keywords
        Token::Word(word) => match word {
            swc_ecma_lexer::token::Word::Keyword(_) => Some(TokenType::Keyword),
            swc_ecma_lexer::token::Word::Null
            | swc_ecma_lexer::token::Word::True
            | swc_ecma_lexer::token::Word::False => Some(TokenType::Keyword),
            swc_ecma_lexer::token::Word::Ident(_) => Some(TokenType::Identifier),
        },

        // Literals
        Token::Str { .. } | Token::Template { .. } => Some(TokenType::String),
        Token::Num { .. } | Token::BigInt { .. } => Some(TokenType::Number),
        Token::Regex(..) => Some(TokenType::Regex),

        // JSX
        Token::JSXTagStart | Token::JSXTagEnd => Some(TokenType::JsxTag),

        // Comments - SWC lexer doesn't emit comments by default
        // We'd need to enable them in lexer config if needed

        // Skip punctuation and other tokens for now
        _ => None,
    }
}

/// Add start and end markers for a token span
fn add_token_markers(markers: &mut Vec<StyleMarker>, span: Span, token_type: TokenType) {
    let start = span.lo.0 as usize;
    let end = span.hi.0 as usize;

    if start < end {
        markers.push(StyleMarker {
            offset: start,
            is_start: true,
            token_type,
        });
        markers.push(StyleMarker {
            offset: end,
            is_start: false,
            token_type,
        });
    }
}

/// Build a map of line numbers to their byte offsets (start, end exclusive)
fn build_line_offset_map(source: &str) -> Vec<(usize, usize)> {
    let mut offsets = Vec::new();
    let mut start = 0;

    for (idx, _) in source.match_indices('\n') {
        offsets.push((start, idx + 1)); // Include the newline
        start = idx + 1;
    }

    // Add the last line if it doesn't end with newline
    if start < source.len() {
        offsets.push((start, source.len()));
    } else if start == source.len() && !source.is_empty() {
        // File ends with newline, add empty last line
        offsets.push((start, start));
    }

    offsets
}

/// Split style markers across line boundaries
/// This ensures we never have a style that spans multiple lines in the marker list
fn split_markers_by_line(
    markers: &[StyleMarker],
    line_offsets: &[(usize, usize)],
) -> Vec<LineHighlight> {
    let mut line_highlights: Vec<LineHighlight> = line_offsets
        .iter()
        .enumerate()
        .map(|(idx, &(start, end))| LineHighlight {
            line: idx + 1,
            line_start_offset: start,
            line_end_offset: end,
            markers: Vec::new(),
        })
        .collect();

    // Track active styles that span lines
    let mut active_styles: Vec<TokenType> = Vec::new();
    let mut marker_idx = 0;

    for line_highlight in &mut line_highlights {
        let line_start = line_highlight.line_start_offset;
        let line_end = line_highlight.line_end_offset;

        // Add start markers for any styles that were active from previous lines
        for &token_type in &active_styles {
            line_highlight.markers.push(StyleMarker {
                offset: 0, // Relative to line start
                is_start: true,
                token_type,
            });
        }

        // Process markers that fall within this line
        while marker_idx < markers.len() && markers[marker_idx].offset < line_end {
            let marker = &markers[marker_idx];

            if marker.offset >= line_start {
                // Marker is within this line
                let relative_offset = marker.offset - line_start;
                line_highlight.markers.push(StyleMarker {
                    offset: relative_offset,
                    is_start: marker.is_start,
                    token_type: marker.token_type,
                });

                // Update active styles
                if marker.is_start {
                    active_styles.push(marker.token_type);
                } else {
                    active_styles.retain(|&t| t != marker.token_type);
                }
            }

            marker_idx += 1;
        }

        // Add end markers for active styles at end of line
        for &token_type in &active_styles {
            let relative_offset = line_end.saturating_sub(line_start);
            line_highlight.markers.push(StyleMarker {
                offset: relative_offset,
                is_start: false,
                token_type,
            });
        }
    }

    line_highlights
}

/// Apply highlights to a line of text
/// Returns the styled text with ANSI codes inserted
pub fn apply_line_highlights(
    line: &str,
    line_highlight: &LineHighlight,
    color_scheme: &ColorScheme,
) -> String {
    if line_highlight.markers.is_empty() {
        return line.to_string();
    }

    let mut result = String::with_capacity(line.len() + line_highlight.markers.len() * 10);
    let mut last_offset = 0;
    let mut active_style: Option<TokenType> = None;

    for marker in &line_highlight.markers {
        // Add any text before this marker
        if marker.offset > last_offset {
            let end = marker.offset.min(line.len());
            result.push_str(&line[last_offset..end]);
            last_offset = end;
        }

        // Apply the style change
        if marker.is_start {
            result.push_str(color_scheme.color_for_token(marker.token_type));
            active_style = Some(marker.token_type);
        } else if active_style == Some(marker.token_type) {
            result.push_str(color_scheme.reset);
            active_style = None;
        }
    }

    // Add any remaining text
    if last_offset < line.len() {
        result.push_str(&line[last_offset..]);
    }

    // Reset at end of line if style is still active
    if active_style.is_some() {
        result.push_str(color_scheme.reset);
    }

    result
}
