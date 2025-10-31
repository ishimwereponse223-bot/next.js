use next_code_frame::{CodeFrameLocation, CodeFrameOptions, Location, render_code_frame};

fn main() {
    let source = r#"// This is a comment
const x = 42; // inline comment
const obj = { foo: 'bar' };
/* Multi-line
   comment */
const result = x > 10 ? 'yes' : 'no';"#;

    let location = CodeFrameLocation {
        start: Location { line: 2, column: 1 },
        end: Some(Location {
            line: 2,
            column: 14, // Mark "const x = 42;"
        }),
    };

    println!("=== With Syntax Highlighting (showing comments and punctuation) ===");
    let options = CodeFrameOptions {
        use_colors: true,
        highlight_code: true,
        ..Default::default()
    };
    let result = render_code_frame(source, &location, &options).unwrap();
    println!("{}", result);
}
