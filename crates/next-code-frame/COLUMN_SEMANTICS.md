# Column Position Semantics

## API Contract

### End Column Semantics: **EXCLUSIVE** (for all error types)

Our implementation follows standard programming range conventions where ranges are represented as `[start, end)`:

- **`start_column`**: **Inclusive** - points to the first character to mark
- **`end_column`**: **EXCLUSIVE** - points one past the last character to mark

### Examples

Given source: `"const x = 123;"`

Positions (1-indexed):

```
const x = 123;
1234567891111111
          01234
```

**Example 1: Mark single character "1"**

```rust
CodeFrameLocation {
    start_column: 11,
    end_column: Some(12),  // Exclusive: stops before 12
}
```

Result: Single `^` at position 11

**Example 2: Mark "123" (3 characters)**

```rust
CodeFrameLocation {
    start_column: 11,
    end_column: Some(14),  // Exclusive: stops before 14, marks 11-13
}
```

Result: `^^^` marking positions 11, 12, 13

**Example 3: Default behavior (no end_column)**

```rust
CodeFrameLocation {
    start_column: 11,
    end_column: None,  // Defaults to start_column + 1 = 12
}
```

Result: Single `^` at position 11 (equivalent to Example 1)

### Rationale

1. **Consistency with SWC**: SWC uses byte-position spans with exclusive end positions `[lo, hi)`
2. **Standard Programming Convention**: Most languages use half-open intervals for ranges (e.g., Rust ranges, Python slices)
3. **Internal Simplicity**: Makes length calculations straightforward: `length = end - start`

### Multiline Error Behavior

For multiline errors, the API semantics remain consistent (exclusive `end_column`), but the rendering behaves appropriately for spanning errors:

```rust
CodeFrameLocation {
    start_line: 2,
    start_column: 3,
    end_line: Some(3),
    end_column: Some(12),  // Exclusive: marks columns 1-11 on last line
}
```

Rendering behavior:

- **First line** (line 2): Marks from `start_column` (3) to end of line
- **Last line** (line 3): Marks from column 1 to `end_column` (exclusive), so columns 1-11

This ensures multiline error spans are visualized correctly, with the first line showing where the error starts and continuing to the end of that line, and the last line showing from the beginning up to (but not including) the `end_column`.

### Compatibility Notes

**Babel-code-frame compatibility**: While babel-code-frame's documentation doesn't explicitly specify inclusive/exclusive semantics, the exclusive end column approach is:

1. More common in programming APIs
2. Consistent with source-level tooling (SWC, Rust compiler errors, etc.)
3. Easier to work with programmatically (avoids off-by-one errors)

**Migration from babel**: If existing code assumes inclusive end columns, you'll need to add 1:

```diff
// Before (if babel used inclusive):
-{ start: { column: 11 }, end: { column: 13 } }  // Marks 11, 12, 13
+{ start: { column: 11 }, end: { column: 14 } }  // Marks 11, 12, 13
```

However, based on our analysis of Next.js's usage patterns, Turbopack already generates exclusive end positions when converting from SWC spans, so no changes should be needed.

### Test Coverage

See `src/tests.rs::test_column_semantics_explicit_end` for verification of these semantics.
