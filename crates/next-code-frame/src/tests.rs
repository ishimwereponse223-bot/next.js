use crate::{CodeFrameLocation, CodeFrameOptions, Location, render_code_frame};

#[test]
fn test_simple_single_line_error() {
    let source = "console.log('hello')";
    let location = CodeFrameLocation {
        start: Location { line: 1, column: 1 },
        end: None,
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
        start: Location { line: 1, column: 1 },
        end: None,
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
        start: Location {
            line: 100,
            column: 1,
        },
        end: None,
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
        start: Location { line: 2, column: 3 },
        end: Some(Location {
            line: 3,
            column: 12,
        }),
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        ..Default::default()
    };

    let result = render_code_frame(source, &location, &options).unwrap();

    // Multiline error shows spanning markers:
    // - Line 2: from column 3 to end of line
    // - Line 3: from start to end_column 12 (exclusive, so marks columns 1-11)
    let expected = r#"  1 | function test() {
> 2 |   console.log('hello')
    |   ^^^^^^^^^^^^^^^^^^^
> 3 |   return 42
    | ^^^^^^^^^^^
  4 | }
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_multiline_error_with_message() {
    let source = "function test() {\n  console.log('hello')\n  return 42\n}";
    let location = CodeFrameLocation {
        start: Location { line: 2, column: 3 },
        end: Some(Location {
            line: 3,
            column: 12,
        }),
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        message: Some("Unexpected expression".to_string()),
        ..Default::default()
    };

    let result = render_code_frame(source, &location, &options).unwrap();

    // Message should only appear once, on the last error line's marker
    // Spanning markers show the full error range
    // end_column: 12 (exclusive) marks columns 1-11
    let expected = r#"  1 | function test() {
> 2 |   console.log('hello')
    |   ^^^^^^^^^^^^^^^^^^^
> 3 |   return 42
    | ^^^^^^^^^^^ Unexpected expression
  4 | }
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_with_message() {
    let source = "const x = 1";
    let location = CodeFrameLocation {
        start: Location { line: 1, column: 7 },
        end: None,
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
        start: Location {
            line: 2,
            column: 250,
        }, // Error in the middle of the long line
        end: None,
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
        start: Location { line: 1, column: 5 },
        end: None,
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
        start: Location {
            line: 1,
            column: 495,
        },
        end: None,
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
        start: Location {
            line: 2,
            column: 250,
        },
        end: None,
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
        start: Location { line: 4, column: 1 },
        end: None,
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
        start: Location {
            line: 99,
            column: 1,
        },
        end: None,
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
        start: Location {
            line: 25000,
            column: 1,
        },
        end: None,
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
        start: Location {
            line: 1,
            column: 100,
        },
        end: Some(Location {
            line: 1,
            column: 400,
        }), // 300 char span
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
        start: Location {
            line: 3,
            column: 25,
        },
        end: None,
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        ..Default::default()
    };

    let result = render_code_frame(source, &location, &options).unwrap();

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

#[test]
fn test_invalid_column_start_out_of_bounds() {
    // Start column beyond line length should be clamped
    let source = "short";
    let location = CodeFrameLocation {
        start: Location {
            line: 1,
            column: 100,
        }, // Way past end of line
        end: None,
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        ..Default::default()
    };

    let result = render_code_frame(source, &location, &options).unwrap();

    // Should clamp to end of line (column 5)
    let expected = r#"> 1 | short
    |      ^
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_invalid_column_end_before_start() {
    // End column before start column should show single marker at start
    let source = "const x = 123;";
    let location = CodeFrameLocation {
        start: Location {
            line: 1,
            column: 11,
        }, // "123"
        end: Some(Location { line: 1, column: 5 }), // Before start - invalid
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        ..Default::default()
    };

    let result = render_code_frame(source, &location, &options).unwrap();

    // Should show single marker at start column (no span)
    let expected = r#"> 1 | const x = 123;
    |           ^
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_invalid_column_both_out_of_bounds() {
    // Both columns out of bounds
    let source = "abc";
    let location = CodeFrameLocation {
        start: Location {
            line: 1,
            column: 10,
        },
        end: Some(Location {
            line: 1,
            column: 20,
        }),
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        ..Default::default()
    };

    let result = render_code_frame(source, &location, &options).unwrap();

    // Both should clamp to line length (3)
    // Since they're equal after clamping, shows single marker
    let expected = r#"> 1 | abc
    |    ^
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_invalid_multiline_end_column_out_of_bounds() {
    // Multiline error with end column out of bounds on last line
    let source = "line1\nshort\nline3";
    let location = CodeFrameLocation {
        start: Location { line: 1, column: 2 },
        end: Some(Location {
            line: 2,
            column: 50,
        }), // Way past end of "short"
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        ..Default::default()
    };

    let result = render_code_frame(source, &location, &options).unwrap();

    // End column 50 clamps to 6 (one past "short" length of 5)
    // Shows spanning markers for multiline error
    let expected = r#"> 1 | line1
    |  ^^^
> 2 | short
    | ^^^^^^
  3 | line3
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_column_semantics_explicit_end() {
    // Test to clarify: is end_column inclusive or exclusive?
    let source = "const x = 123;";

    // Test 1: Mark just the first digit "1" at column 11 (1-indexed)
    // With EXCLUSIVE semantics: end_column should be 12 to mark column 11
    let location = CodeFrameLocation {
        start: Location {
            line: 1,
            column: 11,
        },
        end: Some(Location {
            line: 1,
            column: 12,
        }), // Exclusive: marks [11, 12) = column 11 only
    };
    let options = CodeFrameOptions {
        use_colors: false,
        highlight_code: false,
        ..Default::default()
    };

    let result = render_code_frame(source, &location, &options).unwrap();

    // With exclusive end_column=12, this marks only column 11
    let expected = r#"> 1 | const x = 123;
    |           ^
"#;
    assert_eq!(result, expected);

    // Test 2: Mark "123" which spans columns 11-13 (1-indexed)
    // With EXCLUSIVE semantics: end_column should be 14 to mark columns 11-13
    let location2 = CodeFrameLocation {
        start: Location {
            line: 1,
            column: 11,
        },
        end: Some(Location {
            line: 1,
            column: 14,
        }), // Exclusive: marks [11, 14) = columns 11, 12, 13
    };

    let result2 = render_code_frame(source, &location2, &options).unwrap();

    // With exclusive end_column=14, this marks columns 11-13 = "123"
    let expected2 = r#"> 1 | const x = 123;
    |           ^^^
"#;
    assert_eq!(result2, expected2);
}
