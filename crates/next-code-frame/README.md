# next-code-frame

Fast, scalable code frame rendering for Next.js error reporting, written in Rust.

This crate provides functionality similar to `@babel/code-frame` but with several improvements:
- **Scalability**: Handles arbitrarily large files efficiently
- **Long line handling**: Gracefully scrolls long lines to keep error positions visible and avoid overwhelming the terminal with long lines
- **Syntax highlighting**: Uses SWC lexer for accurate JavaScript/TypeScript tokenization

## Design

Following the `next-taskless` pattern, this crate:
- Has no dependency on turbo-tasks, allowing use in webpack/rspack codepaths
- Is compilable to WASM for environments without native bindings
- Follows "sans-io" patterns - accepts file content as arguments rather than performing IO
   - Modifying it to optionally accept file paths is reasonable future work

## Features

- Terminal width detection with sensible defaults
- Syntax highlighting for JS, TS, JSX, TSX
- Graceful degradation for non-JS files or parsing errors
- ANSI color support matching babel-code-frame aesthetics
- Support for single-line and multi-line error ranges
