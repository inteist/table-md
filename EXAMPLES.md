# Conversion examples

These examples show exact Markdown output. Unless noted otherwise, use:

```sh
table-md < input.txt
```

## Border styles

Each table below converts to the same output:

```markdown
| A   | B   |
| --- | --- |
| 1   | 2   |
```

### ASCII

```text
+---+---+
| A | B |
+---+---+
| 1 | 2 |
+---+---+
```

### Unicode light

```text
┌───┬───┐
│ A │ B │
├───┼───┤
│ 1 │ 2 │
└───┴───┘
```

### Unicode heavy

```text
┏━━━┳━━━┓
┃ A ┃ B ┃
┣━━━╋━━━┫
┃ 1 ┃ 2 ┃
┗━━━┻━━━┛
```

### Unicode double

```text
╔═══╦═══╗
║ A ║ B ║
╠═══╬═══╣
║ 1 ║ 2 ║
╚═══╩═══╝
```

### Unicode rounded

```text
╭───┬───╮
│ A │ B │
├───┼───┤
│ 1 │ 2 │
╰───┴───╯
```

### Unicode dashed

```text
┌┄┄┄┬┄┄┄┐
│ A │ B │
├╌╌╌┼╌╌╌┤
│ 1 │ 2 │
└┈┈┈┴┈┈┈┘
```

### Unicode mixed

Double outer borders and light inner separators:

```text
╔═══╤═══╗
║ A │ B ║
╟───┼───╢
║ 1 │ 2 ║
╚═══╧═══╝
```

## Multiple tables in a document

Input:

```text
# Scores

┌───────┬───────┐
│ Name  │ Score │
├───────┼───────┤
│ Alice │ 42    │
└───────┴───────┘

The next table lists roles.

+-------+--------+
| Name  | Role   |
+-------+--------+
| Bob   | Editor |
+-------+--------+
```

Output:

```markdown
# Scores

| Name  | Score |
| ----- | ----- |
| Alice | 42    |

The next table lists roles.

| Name | Role   |
| ---- | ------ |
| Bob  | Editor |
```

Surrounding headings, prose, blank lines, and trailing spaces are preserved by
the library. The CLI adds a final newline when any table was converted and the
output does not already end in one.

## Multiline cells

Use `--multiline` to join all content lines between horizontal rules into one
logical row:

```sh
table-md --multiline < input.txt
```

Input:

```text
┌────────┬────────┐
│ Name   │ Notes  │
├────────┼────────┤
│ Alice  │ first  │
│ Smith  │ second │
└────────┴────────┘
```

Output:

```markdown
| Name           | Notes           |
| -------------- | --------------- |
| Alice<br>Smith | first<br>second |
```

Separate each logical row, including the header, with a horizontal rule. Empty
fragments are retained in this mode to preserve line alignment.

The default mode also joins continuation lines when the first cell is blank:

Input:

```text
+-------+--------+
| Name  | Notes  |
+-------+--------+
| Alice | first  |
|       | second |
+-------+--------+
```

Output:

```markdown
| Name  | Notes           |
| ----- | --------------- |
| Alice | first<br>second |
```

Use `--single-line` instead if blank first cells represent separate rows rather
than wrapped content.

## Generated headers

Use `--no-header` when every input row is data:

```sh
table-md --no-header < input.txt
```

Input:

```text
┌───────┬────┐
│ Alice │ 42 │
│ Bob   │ 37 │
└───────┴────┘
```

Output:

```markdown
| Column 1 | Column 2 |
| -------- | -------- |
| Alice    | 42       |
| Bob      | 37       |
```

## Literal pipes

Pipes inside Unicode-bordered cells are escaped for Markdown.

Input:

```text
┌───────┬───────┐
│ Name  │ Regex │
├───────┼───────┤
│ Alice │ x | y │
└───────┴───────┘
```

Output:

```markdown
| Name  | Regex  |
| ----- | ------ |
| Alice | x \| y |
```

In ASCII tables, escape literal pipes in the input as `\|` to distinguish them
from column separators. They remain escaped in the output:

```text
+-------+--------+
| Name  | Regex  |
+-------+--------+
| Alice | x \| y |
+-------+--------+
```

## Empty columns

Use `--single-line` to avoid interpreting empty-first-cell rows as
continuations:

```sh
table-md --single-line < input.txt
```

Input:

```text
┌──────┬───────┬──────┐
│ Left │ Name  │ Last │
├──────┼───────┼──────┤
│      │ Alice │      │
│      │ Bob   │      │
└──────┴───────┴──────┘
```

Output:

```markdown
| Left | Name  | Last |
| ---- | ----- | ---- |
|      | Alice |      |
|      | Bob   |      |
```

Empty edge columns are retained, not removed.

## Inconsistent row widths

Short rows are padded to the widest row, including the header.

Input:

```text
│ Name │ Score │
│ Alice │
│ Bob │ 37 │
```

Output on stdout:

```markdown
| Name  | Score |
| ----- | ----- |
| Alice |       |
| Bob   | 37    |
```

Warning on stderr:

```text
table-md: warning: Row 2 has 1 columns; padded to 2.
```

Use `--quiet` to suppress the warning without changing the converted output.
Library callers receive the diagnostic in `ParsedTable.warnings`.

## Existing Markdown tables

An existing pipe table with a Markdown separator row is left unchanged.

Input and output:

```markdown
| Name  | Score |
| :---- | ----: |
| Alice |    42 |
```

If this is the only table in the input, nothing is converted and the CLI
reports:

```text
table-md: warning: No box-table rows were found; input was left unchanged.
```

Use `--quiet` for silent passthrough. The library reports `table_count == 0` and
preserves the input text.
