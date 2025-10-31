use next_code_frame::{CodeFrameLocation, CodeFrameOptions, Location, render_code_frame};

fn main() {
    let source = r#"// Type definition
const greeting: string = "Hello";
const count = 42 + 10;
const regex = /test\d+/gi;

/* Calculate result with
   ternary operator */
const result = count > 10 ? "yes" : "no";

// Object literal
const obj = {
  foo: 'bar',
  baz: true,
};

// Arrow function with JSX
const Component = () => <div>Hello</div>;"#;

    let location = CodeFrameLocation {
        start: Location { line: 8, column: 1 },
        end: Some(Location {
            line: 8,
            column: 43,
        }),
    };

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  Complete Syntax Highlighting Demo (matching Babel)      ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    let options = CodeFrameOptions {
        use_colors: true,
        highlight_code: true,
        ..Default::default()
    };
    let result = render_code_frame(source, &location, &options).unwrap();
    println!("{}", result);

    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  Color Key:                                               ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║  \x1b[36mKeywords\x1b[0m (cyan): const, let, var, if, etc.            ║");
    println!("║  \x1b[33mIdentifiers\x1b[0m (yellow): variable and function names    ║");
    println!("║  \x1b[32mStrings\x1b[0m (green): \"...\", '...', template literals     ║");
    println!("║  \x1b[35mNumbers\x1b[0m (magenta): 42, 0x10, bigints                 ║");
    println!("║  \x1b[33mPunctuation\x1b[0m (yellow): = ; , . : ? + - * /            ║");
    println!("║  \x1b[90mComments\x1b[0m (gray): // and /* */                        ║");
    println!("║  Brackets (default): ( ) [ ] {{ }}                       ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
}
