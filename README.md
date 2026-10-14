# table-md

Convert Unicode and ASCII box tables into Markdown pipe tables. Available as a
reusable Rust library and a CLI that reads from stdin and writes to stdout, with
no external dependencies.

Adapted from the hand-written parser in `table-beave-rs`, without its browser UI
or HTML preview dependencies.

## Example

Input:

```text
┌─────────┬───────┐
│ Name    │ Score │
├─────────┼───────┤
│ Alice   │ 42    │
│ Bob     │ 37    │
└─────────┴───────┘
```

Output:

```markdown
| Name  | Score |
| ----- | ----- |
| Alice | 42    |
| Bob   | 37    |
```

## Features

See [EXAMPLES.md](EXAMPLES.md) for supported border styles and input/output samples.

- Unicode light, heavy, double, rounded, dashed, and mixed border styles.
- ASCII boxes such as `+---+` with `|`-delimited cells.
- Borderless rows enclosed by Unicode bars or ASCII pipes.
- Multiple tables within a document, preserving surrounding text and whitespace.
- Existing pipe tables with Markdown separator rows left unchanged.
- Multiline cells joined with `<br>`.
- Optional generated headers: `Column 1`, `Column 2`, and so on.
- Empty columns retained; inconsistent row widths padded with warnings.
- Pipes within Unicode cells escaped for Markdown; escaped pipes in ASCII cells
  preserved.

## Install and build

Requires Rust 1.85 or newer (edition 2024).

From this repository:

```sh
cargo install --path .
```

Or build without installing:

```sh
cargo build --release
./target/release/table-md < table.txt
```

## CLI usage

```sh
table-md < table.txt
table-md < table.txt > table.md
pbpaste | table-md | pbcopy
```

The clipboard example uses macOS utilities. Any command that emits UTF-8 text
can pipe its output into `table-md`.

To enter a table directly, run `table-md`, paste the text, then finish stdin
with Ctrl-D on Unix/macOS or Ctrl-Z followed by Enter on Windows. Conversion
happens after stdin reaches EOF.

### Options

| Option            | Behavior                                                                              |
| ----------------- | ------------------------------------------------------------------------------------- |
| `--no-header`     | Generate column names and keep the first input row as data.                           |
| `--multiline`     | Join all content lines between horizontal rules into one logical row.                 |
| `--single-line`   | Treat every content line as a separate row, including lines with an empty first cell. |
| `-q`, `--quiet`   | Suppress conversion warnings and the interactive prompt.                              |
| `-h`, `--help`    | Print usage information.                                                              |
| `-V`, `--version` | Print the version.                                                                    |

The first row is used as the header by default. Row grouping defaults to the
automatic heuristic described below. If both row-mode flags are supplied, the
last one takes precedence.

Markdown is written to stdout; prompts, warnings, and errors go to stderr. Input
without recognized table rows passes through unchanged with a warning. Empty or
whitespace-only input does not warn. Warnings do not cause a failure exit
status; invalid arguments, invalid UTF-8, and I/O errors do. A closed output
pipe is treated as a successful exit.

## Library usage

Add a local dependency to your project's `Cargo.toml`, adjusting the path as
needed:

```toml
[dependencies]
table-md = { path = "../table-md", default-features = false }
```

The default `cli` feature enables the executable target. Disable it for a
library-only build; the conversion API remains available.

### Default conversion

```rust
use table_md::convert;

let input = "│ Name │ Score │\n│ Alice │ 42 │";
let result = convert(input);

assert_eq!(
    result.markdown,
    "| Name | Score |\n| --- | --- |\n| Alice | 42 |"
);
assert_eq!(result.table_count, 1);
assert_eq!(result.row_count, 1);
assert_eq!(result.column_count, 2);
assert!(result.warnings.is_empty());
```

### Custom options

```rust
use table_md::{Options, RowMode, convert_with_options};

let input = "│ Alice │ 42 │\n│ Bob │ 37 │";
let result = convert_with_options(
    input,
    Options {
        first_row_is_header: false,
        row_mode: RowMode::SingleLine,
    },
);

assert_eq!(result.row_count, 2);
assert!(result.markdown.starts_with("| Column 1 | Column 2 |"));
```

`ParsedTable` contains:

| Field          | Meaning                                                           |
| -------------- | ----------------------------------------------------------------- |
| `markdown`     | Converted document, including passthrough text.                   |
| `table_count`  | Number of tables converted.                                       |
| `row_count`    | Total body rows across converted tables, excluding headers.       |
| `column_count` | Maximum column count among converted tables.                      |
| `warnings`     | Nonfatal diagnostics, such as padded rows or a header-only table. |

The template-compatible `parse_ascii_table(input, first_row_is_header)` function
is also available. Despite its name, it supports Unicode tables too.

The library preserves whether a converted block ends with a newline. The CLI
adds a final newline to non-newline-terminated output when at least one table
was converted.

## Row grouping and limitations

Wrapped cells and separate rows with empty first columns can look identical.
Choose the mode that matches the source:

- **`RowMode::Auto` (default):** In bordered tables, a content line whose first
  cell is empty continues the previous row. Horizontal rules end a logical row.
  Empty fragments are omitted when joining cells, matching the template's
  heuristic.
- **`RowMode::SingleLine`:** Every content line is a row. Use this when empty
  first cells are actual data rather than continuation markers.
- **`RowMode::Multiline`:** All content lines between horizontal rules form one
  row. Empty fragments are retained to preserve line alignment between cells.
  Separate each logical row, including the header, with a horizontal rule.

Without horizontal rules, each content line is a separate row in every mode.

Additional limitations:

- Merged or spanning cells are not represented; Markdown pipe tables cannot
  express them.
- Unescaped pipes in ASCII rows are column separators. Use `\|` for a literal
  pipe, or use Unicode vertical borders.
- Markdown and HTML within cells are preserved, not sanitized or escaped as
  plain text. Sanitize untrusted output before rendering it as HTML.
- Detection is line-based, not a full Markdown parser. Table-looking content
  inside fenced code blocks is also eligible for conversion.
- Input is read completely into memory; this is not a streaming converter.

## Development

```sh
cargo test --all-features
cargo test --no-default-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

Conversion tests cover border styles, Unicode content, row grouping, empty
cells, pipe escaping, document preservation, and warnings. CLI tests cover
stdin/stdout behavior, options, diagnostics, invalid input, and closed pipes.
