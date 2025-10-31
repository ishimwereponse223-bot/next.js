use anyhow::{Result, bail};

use crate::{highlight::ColorScheme, terminal::get_terminal_width};

/// A source location with line and column
#[derive(Debug, Clone, Copy)]
pub struct Location {
    /// 1-indexed line number
    pub line: usize,
    /// 1-indexed column number (0 means no column highlighting)
    pub column: usize,
}

/// Location information for the error in the source code
///
/// # Column Semantics
///
/// - `start.column`: **Inclusive** - points to the first character to mark
/// - `end.column`: **EXCLUSIVE** - points one past the last character to mark
///
/// This follows standard programming range conventions `[start, end)`.
///
/// Example: To mark "123" at columns 11-13, use:
/// ```ignore
/// CodeFrameLocation {
///     start: Location { line: 1, column: 11 },
///     end: Some(Location { line: 1, column: 14 }),  // Exclusive
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct CodeFrameLocation {
    /// Starting location
    pub start: Location,
    /// Optional ending location
    /// Line is treated inclusively but column is treated exclusively
    pub end: Option<Location>,
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
    /// Whether to attempt syntax highlighting
    pub highlight_code: bool,
    /// Optional message to display with the error
    pub message: Option<String>,
    /// Maximum width for the output (None = terminal width)
    pub max_width: Option<usize>,
}

impl Default for CodeFrameOptions {
    fn default() -> Self {
        Self {
            lines_above: 2,
            lines_below: 3,
            use_colors: true,
            highlight_code: true,
            message: None,
            max_width: None,
        }
    }
}

#[inline]
fn repeat_char_into(s: &mut String, ch: char, count: usize) {
    s.reserve(count);
    for _ in 0..count {
        s.push(ch);
    }
}

fn apply_line_truncation(
    line_content: &str,
    truncation_offset: usize,
    available_code_width: usize,
) -> (String, usize) {
    if truncation_offset > 0 {
        truncate_line(line_content, truncation_offset, available_code_width)
    } else if line_content.len() > available_code_width {
        truncate_line(line_content, 0, available_code_width)
    } else {
        (line_content.to_string(), 0)
    }
}

fn calculate_marker_position(
    location_start_column: usize,
    end_column: usize,
    line_length: usize,
    line_idx: usize,
    start_line: usize,
    end_line: usize,
    column_offset: usize,
    available_code_width: usize,
) -> (usize, usize) {
    // Allow columns to go one past line length (pointing after last char)
    let max_col = line_length + 1;

    // Determine the column range to mark on this line:
    // We use exclusive ranges [start, end) internally
    //
    // API contract: end.column is ALWAYS exclusive (follows [start, end) convention)
    //
    // For rendering:
    // - Single-line: Mark from start.column to end.column (exclusive)
    // - First line of multiline: Mark from start.column to end of line
    // - Last line of multiline: Mark from column 1 to end.column (exclusive)
    let is_single_line_error = start_line == end_line;

    let (range_start, range_end) = if is_single_line_error {
        // Single-line: end.column is exclusive, use directly
        (location_start_column, end_column)
    } else if line_idx == start_line {
        // First line of multiline: mark from start to end of line
        // line_length already represents the last column position
        (location_start_column, line_length)
    } else {
        // Last line of multiline: mark from column 1 to end.column (exclusive)
        (1, end_column)
    };

    // Clamp to reasonable bounds
    let range_start = range_start.min(max_col);
    // Allow small extension past line for off-by-one, but prevent excessive spans
    let reasonable_max = max_col + 1;
    let range_end = range_end.min(reasonable_max);

    // Calculate marker position accounting for truncation
    // When column_offset > 0, visible content is "...XXXXX" where X starts at column_offset
    // Display positions: columns 1-3 are "...", column 4 corresponds to original column_offset
    let marker_col = if column_offset > 0 {
        // Convert original column to display column
        // formula: display_col = (original_col - offset) + 4
        // where 4 accounts for the "..." prefix (3 chars) plus 1-indexing
        if range_start < column_offset {
            // Error starts before visible window, mark from column 4 (after "...")
            4
        } else {
            // Error starts in visible window
            (range_start - column_offset) + 4
        }
    } else {
        // No truncation, use column as-is
        range_start.max(1)
    };

    // If range is invalid (end <= start), show single marker at start
    let marker_length = if range_end > range_start {
        range_end - range_start
    } else {
        1
    };

    // Adjust marker_length if it would extend past available width
    let marker_length = marker_length.min(available_code_width.saturating_sub(marker_col - 1));

    (marker_col, marker_length)
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
    let start_line = location.start.line.saturating_sub(1); // Convert to 0-indexed
    if start_line >= lines.len() {
        return Ok(String::new());
    }

    let end_line = location
        .end
        .map(|l| l.line.saturating_sub(1).min(lines.len() - 1))
        .unwrap_or(start_line);
    if end_line < start_line {
        bail!("source location invalid end_line {end_line} < {start_line}");
    }

    // Normalize end_column:
    // API contract: end.column is always EXCLUSIVE (follows [start, end) convention)
    // - If no end (single-line) and no end.column, default to start.column + 1 (marks 1 char)
    // - If end is set but no end.column, that would be None - but the struct requires it together
    // - Otherwise use the provided end.column as-is
    let end_column = match location.end {
        Some(l) => l.column,
        None => {
            // Single-line error without end defaults to single-char marker
            // end_column = start_column + 1 (exclusive) means mark exactly one character
            if location.start.column > 0 {
                location.start.column + 1
            } else {
                0
            }
        }
    };

    // For rendering, we'll clamp columns to valid ranges per-line

    // Calculate window of lines to show
    let first_line = start_line.saturating_sub(options.lines_above);
    let last_line = (end_line + options.lines_below + 1).min(lines.len());

    // Calculate gutter width (space needed for line numbers)
    let max_line_num = last_line;
    let gutter_width = format!("{}", max_line_num).len();

    let max_width = options.max_width.unwrap_or_else(get_terminal_width);

    // Calculate available width for code (accounting for gutter, markers, and padding)
    // Format: "> N | code" or "  N | code"
    // That's: 1 (marker) + 1 (space) + gutter_width + 3 (" | ")
    let gutter_total_width = 1 + 1 + gutter_width + 3;
    let available_code_width = max_width.saturating_sub(gutter_total_width);
    if available_code_width == 0 {
        bail!("max_width {max_width} too small to render a code frame")
    }

    // Calculate truncation offset for long lines - only if any line actually needs it
    // Center the error range if any line in the error range needs truncation
    let truncation_offset = calculate_truncation_offset(
        &lines,
        first_line,
        last_line,
        start_line,
        location.start.column,
        end_column,
        available_code_width,
    );

    let color_scheme = if options.use_colors {
        ColorScheme::colored()
    } else {
        ColorScheme::plain()
    };
    let mut output = String::new();

    // Add message if provided and no column specified
    if let Some(ref message) = options.message
        && location.start.column == 0
    {
        repeat_char_into(&mut output, ' ', gutter_total_width);
        output.push_str(color_scheme.marker);
        output.push_str(message);
        output.push_str(color_scheme.reset);
        output.push('\n');
    }

    // Render each line
    for (line_idx, line_content) in lines.iter().enumerate().take(last_line).skip(first_line) {
        let is_error_line = line_idx >= start_line && line_idx <= end_line;
        let line_num = line_idx + 1;

        // Apply consistent truncation to all lines
        let (visible_content, column_offset) =
            apply_line_truncation(line_content, truncation_offset, available_code_width);

        // Line prefix with number
        if is_error_line {
            output.push_str(color_scheme.marker);
            output.push('>');
            output.push_str(color_scheme.reset);
        } else {
            output.push(' ');
        }
        output.push(' ');
        output.push_str(color_scheme.gutter);
        output.push_str(&format!("{:>width$}", line_num, width = gutter_width));
        output.push_str(color_scheme.reset);
        output.push_str(color_scheme.gutter);
        output.push_str(" |");
        output.push_str(color_scheme.reset);

        // Line content (with space separator if not empty)
        if !visible_content.is_empty() {
            output.push(' ');
            output.push_str(&visible_content);
        }
        output.push('\n');

        // Add marker line if this is an error line with column info
        if is_error_line && location.start.column > 0 {
            let (marker_col, marker_length) = calculate_marker_position(
                location.start.column,
                end_column,
                line_content.len(),
                line_idx,
                start_line,
                end_line,
                column_offset,
                available_code_width,
            );

            output.push(' ');
            output.push(' ');
            output.push_str(color_scheme.gutter);
            output.push_str(&format!("{:>width$} |", "", width = gutter_width));
            output.push_str(color_scheme.reset);
            output.push(' ');
            repeat_char_into(&mut output, ' ', marker_col - 1);
            output.push_str(color_scheme.marker);
            repeat_char_into(&mut output, '^', marker_length);
            output.push_str(color_scheme.reset);

            // Add message only on the last error line's marker
            if line_idx == end_line
                && let Some(ref message) = options.message
            {
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
const ELLIPSIS: &str = "...";
const ELLIPSIS_LEN: usize = 3;

/// Calculate the truncation offset for all lines in the window.
/// This ensures all lines are "scrolled" to the same position, centering the error range.
fn calculate_truncation_offset(
    lines: &[&str],
    first_line: usize,
    last_line: usize,
    _error_line: usize,
    start_column: usize,
    end_column: usize,
    available_width: usize,
) -> usize {
    // Check if any line in the window needs truncation
    let needs_truncation = (first_line..last_line).any(|i| lines[i].len() > available_width);

    if !needs_truncation {
        return 0;
    }

    // If we need truncation, center the error range
    // We need to account for the "..." ellipsis (3 chars) on each side
    let available_with_ellipsis = available_width.saturating_sub(2 * ELLIPSIS_LEN);

    if start_column == 0 {
        // No specific error column, start at beginning
        return 0;
    }

    // Calculate the midpoint of the error range
    // end_column is exclusive, so the range is [start_column, end_column)
    let start_0idx = start_column.saturating_sub(1);
    let end_0idx = end_column.saturating_sub(1);
    let error_midpoint = (start_0idx + end_0idx) / 2;

    // Try to center the error range midpoint
    let half_width = available_with_ellipsis / 2;

    if error_midpoint < half_width {
        // Error range is near the start, start from beginning
        0
    } else {
        // Center the error range midpoint
        error_midpoint.saturating_sub(half_width)
    }
}

/// Truncate a line at a specific offset, adding ellipsis as needed
fn truncate_line(line: &str, offset: usize, max_width: usize) -> (String, usize) {
    // If no offset and line fits, return as-is
    if offset == 0 && line.len() <= max_width {
        return (line.to_string(), 0);
    }

    let mut result = String::with_capacity(max_width);
    let actual_offset = offset;

    // Add leading ellipsis if we're starting mid-line
    if offset > 0 {
        result.push_str(ELLIPSIS);
    }

    // Calculate how much content we can show
    let available_content_width = if offset > 0 {
        max_width.saturating_sub(ELLIPSIS_LEN)
    } else {
        max_width
    };

    // Check if line would extend past the end
    let remaining_line = if offset < line.len() {
        &line[offset..]
    } else {
        // Offset is past line length - show just ellipsis
        return (ELLIPSIS.to_string(), offset);
    };

    let needs_trailing_ellipsis = remaining_line.len() > available_content_width;
    let content_width = if needs_trailing_ellipsis {
        available_content_width.saturating_sub(ELLIPSIS_LEN)
    } else {
        available_content_width.min(remaining_line.len())
    };

    // Extract the visible portion, being careful about UTF-8 boundaries
    let visible_end = remaining_line
        .char_indices()
        .take(content_width)
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);

    result.push_str(&remaining_line[..visible_end]);

    if needs_trailing_ellipsis {
        result.push_str(ELLIPSIS);
    }

    (result, actual_offset)
}
