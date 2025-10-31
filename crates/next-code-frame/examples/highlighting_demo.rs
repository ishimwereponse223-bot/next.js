use next_code_frame::{CodeFrameLocation, CodeFrameOptions, Location, render_code_frame};

fn main() {
    let source = r#"const greeting = "Hello, world!";
const number = 42;
const regex = /test/g;
const obj = { foo: 'bar' };"#;

    let location = CodeFrameLocation {
        start: Location {
            line: 2,
            column: 16,
        },
        end: Some(Location {
            line: 2,
            column: 18, // Mark "42"
        }),
    };

    // Without highlighting
    println!("=== Without Highlighting ===");
    let options = CodeFrameOptions {
        use_colors: true,
        highlight_code: false,
        ..Default::default()
    };
    let result = render_code_frame(source, &location, &options).unwrap();
    println!("{}", result);

    // With highlighting
    println!("\n=== With Syntax Highlighting ===");
    let options = CodeFrameOptions {
        use_colors: true,
        highlight_code: true,
        ..Default::default()
    };
    let result = render_code_frame(source, &location, &options).unwrap();
    println!("{}", result);
}
