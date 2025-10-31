/// Token types for syntax highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Will be used in Phase 4
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
#[allow(dead_code)] // Fields will be used in Phase 4
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
    #[allow(dead_code)] // Will be used in Phase 4
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

/// Placeholder for syntax highlighting - will be implemented in Phase 4
pub fn highlight_code(_source: &str, _use_colors: bool) -> String {
    // For Phase 2, we just return the source as-is
    // This will be replaced with SWC-based highlighting in Phase 4
    _source.to_string()
}
