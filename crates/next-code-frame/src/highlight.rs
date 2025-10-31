use oxc_allocator::Allocator;
use oxc_ast::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

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

    // Create allocator for OXC
    let allocator = Allocator::default();

    // Configure source type for TypeScript/JSX
    let source_type = SourceType::from_path("file.tsx").unwrap_or_default();

    // Parse with OXC
    let parser_ret = Parser::new(&allocator, source, source_type).parse();

    // Visit AST and extract highlights
    let mut visitor = HighlightVisitor {
        markers: Vec::new(),
    };

    visitor.visit_program(&parser_ret.program);

    // Sort markers by offset
    visitor.markers.sort();

    // Split markers into per-line highlights
    split_markers_by_line(&visitor.markers, &line_offsets)
}

/// Visitor that extracts style markers from the AST
struct HighlightVisitor {
    markers: Vec<StyleMarker>,
}

impl HighlightVisitor {
    fn add_span(&mut self, span: Span, token_type: TokenType) {
        let start = span.start as usize;
        let end = span.end as usize;

        if start < end {
            self.markers.push(StyleMarker {
                offset: start,
                is_start: true,
                token_type,
            });
            self.markers.push(StyleMarker {
                offset: end,
                is_start: false,
                token_type,
            });
        }
    }
}

impl<'a> Visit<'a> for HighlightVisitor {
    fn visit_string_literal(&mut self, lit: &oxc_ast::ast::StringLiteral<'a>) {
        self.add_span(lit.span, TokenType::String);
    }

    fn visit_numeric_literal(&mut self, lit: &oxc_ast::ast::NumericLiteral<'a>) {
        self.add_span(lit.span, TokenType::Number);
    }

    fn visit_big_int_literal(&mut self, lit: &oxc_ast::ast::BigIntLiteral<'a>) {
        self.add_span(lit.span, TokenType::Number);
    }

    fn visit_reg_exp_literal(&mut self, lit: &oxc_ast::ast::RegExpLiteral<'a>) {
        self.add_span(lit.span, TokenType::Regex);
    }

    fn visit_template_literal(&mut self, lit: &oxc_ast::ast::TemplateLiteral<'a>) {
        // Highlight the entire template literal
        self.add_span(lit.span, TokenType::String);
    }

    // Visit keywords through specific statement/expression types
    fn visit_variable_declaration(&mut self, decl: &oxc_ast::ast::VariableDeclaration<'a>) {
        // Highlight the keyword (var, let, const)
        // We can infer the keyword span from the start of the declaration
        // For simplicity, we'll skip keyword highlighting for now and focus on literals
        oxc_ast::visit::walk::walk_variable_declaration(self, decl);
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
