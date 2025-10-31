use anyhow::{Result, bail};

use crate::{
    highlight::{ColorScheme, highlight_code},
    terminal::get_terminal_width,
};

/// Location information for the error in the source code
#[derive(Debug, Clone)]
pub struct CodeFrameLocation {
    /// Starting line (1-indexed)
    pub start_line: usize,
    /// Starting column (1-indexed, 0 means no column highlighting)
    pub start_column: usize,
    /// Optional ending line (1-indexed)
    pub end_line: Option<usize>,
    /// Optional ending column (1-indexed)
    pub end_column: Option<usize>,
}

/// Options for rendering the code frame
#[derive(Debug, Clone)]
pub struct CodeFrameOptions {
    /// Number of lines to show before the error
    pub lines_above: usize,
    /// Number of lines to show after the error
    pub lines_below: usize,
    /// Whether to use color output
    pub use_colors: bool,
    /// Whether to highlight code syntax
    pub highlight_code: bool,
    /// Optional message to display
    pub message: Option<String>,
    /// Maximum width for output (None = auto-detect)
    pub max_width: Option<usize>,
}

impl Default for CodeFrameOptions {
    fn default() -> Self {
        Self {
            lines_above: 2,
            lines_below: 3,
            use_colors: false,
            highlight_code: true,
            message: None,
            max_width: None,
        }
    }
}

/// Renders a code frame showing the location of an error in source code
pub fn render_code_frame(
    source: &str,
    location: &CodeFrameLocation,
    options: &CodeFrameOptions,
) -> Result<String> {
    if source.is_empty() {
        return Ok(String::new());
    }
    // Split source into lines
    let lines: Vec<&str> = source.lines().collect();

    // Validate location
    let start_line = location.start_line.saturating_sub(1); // Convert to 0-indexed
    if start_line >= lines.len() {
        return Ok(String::new());
    }

    let end_line = location
        .end_line
        .map(|l| l.saturating_sub(1).min(lines.len() - 1))
        .unwrap_or(start_line);
    if end_line < start_line {
        bail!("source location invalid end_line {end_line} < {start_line}");
    }

    // Calculate window of lines to show
    let first_line = start_line.saturating_sub(options.lines_above);
    let last_line = (end_line + options.lines_below + 1).min(lines.len());

    // Calculate gutter width (space needed for line numbers)
    let max_line_num = last_line;
    let gutter_width = format!("{}", max_line_num).len();

    let max_width = options.max_width.unwrap_or_else(get_terminal_width);

    // Calculate available width for code (accounting for gutter, markers, and padding)
    // Format: " > N | code" or "   N | code"
    // That's: 1 (space) + 1 (marker) + gutter_width + 3 (" | ")
    let gutter_total_width = 1 + 1 + gutter_width + 3;
    let available_code_width = max_width.saturating_sub(gutter_total_width);
    if available_code_width == 0 {
        bail!("max_width {max_width} too small to render a code frame")
    }

    // Calculate truncation offset for long lines - only if any line actually needs it
    // Center the error column if any line in the error range needs truncation
    let truncation_offset = calculate_truncation_offset(
        &lines,
        first_line,
        last_line,
        start_line,
        location.start_column,
        available_code_width,
    );

    let color_scheme = if options.use_colors {
        ColorScheme::colored()
    } else {
        ColorScheme::plain()
    };
    let mut output = String::new();

    // Add message if provided and no column specified
    if let Some(ref message) = options.message {
        if location.start_column == 0 {
            output.push_str(&format!(
                "{}{}{}{}\n",
                " ".repeat(gutter_total_width),
                color_scheme.marker,
                message,
                color_scheme.reset
            ));
        }
    }

    // Render each line
    for line_idx in first_line..last_line {
        let line_num = line_idx + 1;
        let line_content = lines[line_idx];
        let is_error_line = line_idx >= start_line && line_idx <= end_line;

        // Apply consistent truncation to all lines in the window
        // All lines scroll together with the same offset
        let (visible_content, column_offset) = if truncation_offset > 0 {
            truncate_line(line_content, truncation_offset, available_code_width)
        } else if line_content.len() > available_code_width {
            // No global offset, but this specific line is too long
            truncate_line(line_content, 0, available_code_width)
        } else {
            (line_content.to_string(), 0)
        };

        // Highlight code if requested
        let displayed_content = if options.highlight_code {
            highlight_code(&visible_content, options.use_colors)
        } else {
            visible_content
        };

        // Render line with gutter
        let marker = if is_error_line { ">" } else { " " };

        output.push_str(color_scheme.reset);
        if is_error_line {
            output.push_str(color_scheme.marker);
        }
        output.push_str(marker);
        output.push_str(color_scheme.gutter);
        output.push_str(&format!(" {:>width$} |", line_num, width = gutter_width));
        output.push_str(color_scheme.reset);

        if !displayed_content.is_empty() {
            output.push(' ');
            output.push_str(&displayed_content);
        }

        output.push_str(color_scheme.reset);
        output.push('\n');

        // Add marker line with ^ if this is the error line and we have a column
        if is_error_line
            && line_idx == start_line
            && location.start_column > 0
            && location.start_column <= line_content.len()
        {
            // Calculate marker position and length accounting for column offset
            // If we've truncated from the left, adjust positions relative to visible content
            let marker_col = if column_offset > 0 {
                // We have truncation, adjust for it
                if location.start_column > column_offset {
                    // Start column is visible after truncation
                    location.start_column - column_offset + 1
                } else {
                    // Start column was truncated away, marker starts at beginning
                    1
                }
            } else {
                // No truncation, use original column
                location.start_column
            };

            // Calculate marker length, considering truncation and available space
            let marker_length = if let Some(end_col) = location.end_column {
                if line_idx == end_line && end_col > location.start_column {
                    // Calculate visible span within the truncated line
                    let visible_start = location.start_column.max(column_offset);
                    let visible_end = end_col.min(column_offset + available_code_width);

                    if visible_end > visible_start {
                        // Length of visible span
                        let visible_span = visible_end - visible_start;
                        // Make sure we don't exceed available width
                        visible_span.min(available_code_width - (marker_col - 1))
                    } else {
                        1
                    }
                } else {
                    1
                }
            } else {
                1
            };

            output.push_str(color_scheme.reset);
            output.push_str(" "); // Marker column (space instead of >)
            output.push_str(color_scheme.gutter);
            output.push_str(&format!(" {:>width$} |", "", width = gutter_width));
            output.push_str(color_scheme.reset);
            output.push(' ');
            output.push_str(&" ".repeat(marker_col - 1));
            output.push_str(color_scheme.marker);
            output.push_str(&"^".repeat(marker_length));
            output.push_str(color_scheme.reset);

            // Add message on marker line if provided
            if let Some(ref message) = options.message {
                output.push(' ');
                output.push_str(color_scheme.marker);
                output.push_str(message);
                output.push_str(color_scheme.reset);
            }

            output.push('\n');
        }
    }

    Ok(output)
}

/// Calculate the truncation offset for all lines in the window.
/// This ensures all lines are "scrolled" to the same position.
fn calculate_truncation_offset(
    lines: &[&str],
    first_line: usize,
    last_line: usize,
    _error_line: usize,
    error_column: usize,
    available_width: usize,
) -> usize {
    // Check if any line in the window needs truncation
    let needs_truncation = (first_line..last_line).any(|i| lines[i].len() > available_width);

    if !needs_truncation {
        return 0;
    }

    // If we need truncation, center the error column
    // We need to account for the "..." ellipsis (3 chars) on each side
    const ELLIPSIS_LEN: usize = 3;
    let available_with_ellipsis = available_width.saturating_sub(2 * ELLIPSIS_LEN);

    if error_column == 0 {
        // No specific error column, start at beginning
        return 0;
    }

    // Try to center the error column
    let half_width = available_with_ellipsis / 2;
    let error_col_0idx = error_column.saturating_sub(1);

    if error_col_0idx < half_width {
        // Error is near the start, start from beginning
        0
    } else {
        // Center the error column
        error_col_0idx.saturating_sub(half_width)
    }
}

/// Truncate a line at a specific offset, adding ellipsis as needed
fn truncate_line(line: &str, offset: usize, max_width: usize) -> (String, usize) {
    const ELLIPSIS: &str = "...";
    const ELLIPSIS_LEN: usize = 3;

    // If no offset and line fits, return as-is
    if offset == 0 && line.len() <= max_width {
        return (line.to_string(), 0);
    }

    // If offset is 0 but line is too long, we only need end ellipsis
    if offset == 0 {
        let content_width = max_width.saturating_sub(ELLIPSIS_LEN);
        let end_idx = line
            .char_indices()
            .nth(content_width)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        return (format!("{}{}", &line[..end_idx], ELLIPSIS), 0);
    }

    // We have an offset, so we need start ellipsis
    let needs_start_ellipsis = true;
    let mut available_width = max_width.saturating_sub(ELLIPSIS_LEN);

    // Find start position at character boundary
    let start_idx = line
        .char_indices()
        .find(|(i, _)| *i >= offset)
        .map(|(i, _)| i)
        .unwrap_or(line.len());

    // If the line is completely scrolled out (offset beyond line length), show just ellipsis
    if start_idx >= line.len() {
        return (ELLIPSIS.to_string(), offset);
    }

    // Check if we need end ellipsis
    let remaining_chars = line[start_idx..].len();
    let needs_end_ellipsis = remaining_chars > available_width;

    if needs_end_ellipsis {
        available_width = available_width.saturating_sub(ELLIPSIS_LEN);
    }

    // Extract content
    let end_target = start_idx + available_width;
    let end_idx = line
        .char_indices()
        .skip_while(|(i, _)| *i < start_idx)
        .take_while(|(i, _)| *i < end_target)
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(start_idx.min(line.len()));

    let content = &line[start_idx..end_idx];

    let result = match (needs_start_ellipsis, needs_end_ellipsis) {
        (true, true) => format!("{}{}{}", ELLIPSIS, content, ELLIPSIS),
        (true, false) => format!("{}{}", ELLIPSIS, content),
        (false, true) => format!("{}{}", content, ELLIPSIS),
        (false, false) => content.to_string(),
    };

    (result, start_idx)
}
