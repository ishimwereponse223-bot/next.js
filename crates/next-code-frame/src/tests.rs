use crate::{CodeFrameLocation, CodeFrameOptions, render_code_frame};

#[test]
fn test_simple_single_line_error() {
    let source = "console.log('hello')";
    let location = CodeFrameLocation {
        start_line: 1,
        start_column: 1,
        end_line: None,
        end_column: None,
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        ..Default::default()
    };

    let result = render_code_frame(source, &location, &options).unwrap();

    let expected = r#"> 1 | console.log('hello')
    | ^
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_empty_source() {
    let source = "";
    let location = CodeFrameLocation {
        start_line: 1,
        start_column: 1,
        end_line: None,
        end_column: None,
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        ..Default::default()
    };

    let result = render_code_frame(source, &location, &options).unwrap();

    let expected = "";
    assert_eq!(result, expected);
}

#[test]
fn test_invalid_line_number() {
    let source = "line 1\nline 2";
    let location = CodeFrameLocation {
        start_line: 100,
        start_column: 1,
        end_line: None,
        end_column: None,
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        ..Default::default()
    };

    let result = render_code_frame(source, &location, &options).unwrap();

    let expected = "";
    assert_eq!(result, expected);
}

#[test]
fn test_multiline_error() {
    let source = "function test() {\n  console.log('hello')\n  return 42\n}";
    let location = CodeFrameLocation {
        start_line: 2,
        start_column: 3,
        end_line: Some(3),
        end_column: Some(12),
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        ..Default::default()
    };

    let result = render_code_frame(source, &location, &options).unwrap();

    let expected = r#"  1 | function test() {
> 2 |   console.log('hello')
    |   ^
> 3 |   return 42
  4 | }
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_with_message() {
    let source = "const x = 1";
    let location = CodeFrameLocation {
        start_line: 1,
        start_column: 7,
        end_line: None,
        end_column: None,
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        message: Some("Expected semicolon".to_string()),
        ..Default::default()
    };

    let result = render_code_frame(source, &location, &options).unwrap();

    let expected = r#"> 1 | const x = 1
    |       ^ Expected semicolon
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_long_line_single_error() {
    // Create a very long line with error in the middle
    let long_line = "a".repeat(500);
    let source = format!("short\n{}\nshort", long_line);

    let location = CodeFrameLocation {
        start_line: 2,
        start_column: 250, // Error in the middle of the long line
        end_line: None,
        end_column: None,
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        max_width: Some(100),
        ..Default::default()
    };

    let result = render_code_frame(&source, &location, &options).unwrap();

    // All lines should be truncated at the same offset (centered on error)
    // Short lines are completely scrolled out and show only "..."
    let expected = r#"  1 | ...
> 2 | ...aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa...
    |                                              ^
  3 | ...
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_long_line_at_start() {
    // Error at the beginning of a long line
    let long_line = "a".repeat(500);
    let source = long_line.clone();

    let location = CodeFrameLocation {
        start_line: 1,
        start_column: 5,
        end_line: None,
        end_column: None,
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        max_width: Some(100),
        ..Default::default()
    };

    let result = render_code_frame(&source, &location, &options).unwrap();

    // Should only have ellipsis at the end
    // Error is at column 5, so marker should be at position 5 (4 spaces)
    let expected = r#"> 1 | aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa...
    |     ^
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_long_line_at_end() {
    // Error at the end of a long line
    let long_line = "a".repeat(500);
    let source = long_line.clone();

    let location = CodeFrameLocation {
        start_line: 1,
        start_column: 495,
        end_line: None,
        end_column: None,
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        max_width: Some(100),
        ..Default::default()
    };

    let result = render_code_frame(&source, &location, &options).unwrap();

    // Should only have ellipsis at the start
    // The marker position needs to account for the truncation offset
    let expected = r#"> 1 | ...aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    |                                              ^
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_long_line_multiline_aligned() {
    // Multiple long lines should all be truncated at the same offset
    let long_line1 = "b".repeat(500);
    let long_line2 = "c".repeat(500);
    let long_line3 = "d".repeat(500);
    let source = format!("{}\n{}\n{}", long_line1, long_line2, long_line3);

    let location = CodeFrameLocation {
        start_line: 2,
        start_column: 250,
        end_line: None,
        end_column: None,
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        max_width: Some(100),
        lines_above: 1,
        lines_below: 1,
        ..Default::default()
    };

    let result = render_code_frame(&source, &location, &options).unwrap();

    // All lines truncated at same position
    let expected = r#"  1 | ...bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb...
> 2 | ...cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc...
    |                                              ^
  3 | ...dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd...
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_context_lines() {
    let source = "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7";
    let location = CodeFrameLocation {
        start_line: 4,
        start_column: 1,
        end_line: None,
        end_column: None,
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        lines_above: 2,
        lines_below: 2,
        ..Default::default()
    };

    let result = render_code_frame(source, &location, &options).unwrap();

    let expected = r#"  2 | line 2
  3 | line 3
> 4 | line 4
    | ^
  5 | line 5
  6 | line 6
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_gutter_width_alignment() {
    let source = (1..=100)
        .map(|i| format!("line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let location = CodeFrameLocation {
        start_line: 99,
        start_column: 1,
        end_line: None,
        end_column: None,
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        lines_above: 2,
        lines_below: 1,
        ..Default::default()
    };

    let result = render_code_frame(&source, &location, &options).unwrap();

    let expected = r#"   97 | line 97
   98 | line 98
>  99 | line 99
      | ^
  100 | line 100
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_large_file() {
    // Test with a multi-megabyte file
    let line = "x".repeat(100);
    let lines: Vec<String> = (1..=50000)
        .map(|i| format!("line {} {}", i, line))
        .collect();
    let source = lines.join("\n");

    let location = CodeFrameLocation {
        start_line: 25000,
        start_column: 1,
        end_line: None,
        end_column: None,
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        lines_above: 2,
        lines_below: 2,
        ..Default::default()
    };

    let result = render_code_frame(&source, &location, &options).unwrap();

    let expected = format!(
        r#"  24998 | line 24998 {}
  24999 | line 24999 {}
> 25000 | line 25000 {}
        | ^
  25001 | line 25001 {}
  25002 | line 25002 {}
"#,
        line, line, line, line, line
    );
    assert_eq!(result, expected);
}

#[test]
fn test_long_error_span() {
    // Test error span that is longer than available width
    let long_line = "a".repeat(500);
    let source = long_line.clone();

    let location = CodeFrameLocation {
        start_line: 1,
        start_column: 100,
        end_line: None,
        end_column: Some(400), // 300 char span
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        max_width: Some(100),
        ..Default::default()
    };

    let result = render_code_frame(&source, &location, &options).unwrap();

    // The span is 300 chars but we can only show ~87 chars
    // The code is centered on the span, so the marker shows the visible portion
    // Marker starts at position relative to the visible content
    let expected = r#"> 1 | ...aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa...
    |                                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_markdown_file() {
    // Markdown file should not crash (no syntax highlighting)
    let source = r#"# Title

This is a paragraph with some **bold** text.

```javascript
const x = 1;
```

Another paragraph.
"#;

    let location = CodeFrameLocation {
        start_line: 3,
        start_column: 25,
        end_line: None,
        end_column: None,
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        ..Default::default()
    };

    let result = render_code_frame(&source, &location, &options).unwrap();

    let expected = r#"  1 | # Title
  2 |
> 3 | This is a paragraph with some **bold** text.
    |                         ^
  4 |
  5 | ```javascript
  6 | const x = 1;
"#;
    assert_eq!(result, expected);
}
