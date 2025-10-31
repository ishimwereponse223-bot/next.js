use insta::assert_snapshot;

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
    assert_snapshot!(result, @r"
    > 1 | console.log('hello')
        | ^
    ");
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
    assert_snapshot!(result, @"");
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
    assert_snapshot!(result, @"");
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
    assert_snapshot!(result, @r"
      1 | function test() {
    > 2 |   console.log('hello')
        |   ^^^^^^^^^^^^^^^^^^^
    > 3 |   return 42
        | ^^^^^^^^^^^
      4 | }
    ");
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
    assert_snapshot!(result, @r"
      1 | function test() {
    > 2 |   console.log('hello')
        |   ^^^^^^^^^^^^^^^^^^^
    > 3 |   return 42
        | ^^^^^^^^^^^ Unexpected expression
      4 | }
    ");
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
    assert_snapshot!(result, @r"
    > 1 | const x = 1
        |       ^ Expected semicolon
    ");
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
    assert_snapshot!(result, @r"
      1 | ...
    > 2 | ...aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa...
        |                                                 ^
      3 | ...
    ");
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
    assert_snapshot!(result, @r"
    > 1 | aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa...
        |     ^
    ");
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
    assert_snapshot!(result, @r"
    > 1 | ...aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
        |                                                 ^
    ");
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
    assert_snapshot!(result, @r"
      1 | ...bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb...
    > 2 | ...cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc...
        |                                                 ^
      3 | ...dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd...
    ");
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
    assert_snapshot!(result, @r"
      2 | line 2
      3 | line 3
    > 4 | line 4
        | ^
      5 | line 5
      6 | line 6
    ");
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
    assert_snapshot!(result, @r"
       97 | line 97
       98 | line 98
    >  99 | line 99
          | ^
      100 | line 100
    ");
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
    assert_snapshot!(result, @r"
      24998 | line 24998 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
      24999 | line 24999 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    > 25000 | line 25000 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
            | ^
      25001 | line 25001 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
      25002 | line 25002 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    ");
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
    assert_snapshot!(result, @r"
    > 1 | ...aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa...
        |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    ");
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
    assert_snapshot!(result, @r"
      1 | # Title
      2 |
    > 3 | This is a paragraph with some **bold** text.
        |                         ^
      4 |
      5 | ```javascript
      6 | const x = 1;
    ");
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
    assert_snapshot!(result, @r"
    > 1 | short
        |      ^
    ");
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
    assert_snapshot!(result, @r"
    > 1 | const x = 123;
        |           ^
    ");
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
    assert_snapshot!(result, @r"
    > 1 | abc
        |    ^
    ");
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
    assert_snapshot!(result, @r"
    > 1 | line1
        |  ^^^
    > 2 | short
        | ^^^^^^
      3 | line3
    ");
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
    assert_snapshot!(result, @r"
    > 1 | const x = 123;
        |           ^
    ");

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

    let result = render_code_frame(source, &location2, &options).unwrap();
    assert_snapshot!(result, @r"
    > 1 | const x = 123;
        |           ^^^
    ");
}
