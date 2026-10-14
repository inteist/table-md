use table_md::{Options, RowMode, convert, convert_with_options, parse_ascii_table};

#[test]
fn converts_ascii_table_with_unseparated_body_rows() {
    let input = "+------+-------+\n| Name | Value |\n+------+-------+\n| A    | 1     |\n| B    | 2     |\n+------+-------+";
    let result = convert(input);
    assert_eq!(
        result.markdown,
        "| Name | Value |\n| --- | --- |\n| A | 1 |\n| B | 2 |"
    );
    assert_eq!(result.table_count, 1);
    assert_eq!(result.row_count, 2);
    assert_eq!(result.column_count, 2);
    assert!(result.warnings.is_empty());
}

#[test]
fn converts_unicode_border_styles() {
    let cases = [
        "┌───┬───┐\n│ A │ B │\n├───┼───┤\n│ 1 │ 2 │\n└───┴───┘",
        "┏━━━┳━━━┓\n┃ A ┃ B ┃\n┣━━━╋━━━┫\n┃ 1 ┃ 2 ┃\n┗━━━┻━━━┛",
        "╔═══╦═══╗\n║ A ║ B ║\n╠═══╬═══╣\n║ 1 ║ 2 ║\n╚═══╩═══╝",
        "╭───┬───╮\n│ A │ B │\n├───┼───┤\n│ 1 │ 2 │\n╰───┴───╯",
        "╔═══╤═══╗\n║ A ║ B ║\n╟───┼───╢\n║ 1 ║ 2 ║\n╚═══╧═══╝",
        "┌┄┄┄┬┄┄┄┐\n│ A │ B │\n├╌╌╌┼╌╌╌┤\n│ 1 │ 2 │\n└┈┈┈┴┈┈┈┘",
    ];
    for input in cases {
        let result = convert(input);
        assert_eq!(
            result.markdown, "| A | B |\n| --- | --- |\n| 1 | 2 |",
            "{input}"
        );
        assert_eq!(result.row_count, 1);
    }
}

#[test]
fn preserves_unicode_cell_content_and_accepts_indentation() {
    let input = "  ┌────┬────┐\n  │ 행성 │ 분류 │\n  ├────┼────┤\n  │ ① 지구 │ 암석형 · 생명체 🌍 │\n  └────┴────┘";
    assert_eq!(
        convert(input).markdown,
        "| 행성 | 분류 |\n| --- | --- |\n| ① 지구 | 암석형 · 생명체 🌍 |"
    );
}

#[test]
fn generates_headers_for_each_table() {
    let input = "│ A │ 1 │\n\n│ B │ 2 │";
    let result = parse_ascii_table(input, false);
    assert_eq!(
        result.markdown,
        "| Column 1 | Column 2 |\n| --- | --- |\n| A | 1 |\n\n| Column 1 | Column 2 |\n| --- | --- |\n| B | 2 |"
    );
    assert_eq!(result.table_count, 2);
    assert_eq!(result.row_count, 2);
}

#[test]
fn joins_blank_first_cell_continuations() {
    let input = "+------+-------+\n| Name | Notes |\n+------+-------+\n| A    | one   |\n|      | two   |\n+------+-------+";
    assert_eq!(
        convert(input).markdown,
        "| Name | Notes |\n| --- | --- |\n| A | one<br>two |"
    );
}

#[test]
fn multiline_option_joins_nonempty_first_cells_between_rules() {
    let input = "┌───┬───┐\n│ Name │ Notes │\n├───┼───┤\n│ first │ one │\n│ second │ two │\n├───┼───┤\n│ next │ three │\n└───┴───┘";
    let result = convert_with_options(
        input,
        Options {
            row_mode: RowMode::Multiline,
            ..Options::default()
        },
    );
    assert_eq!(
        result.markdown,
        "| Name | Notes |\n| --- | --- |\n| first<br>second | one<br>two |\n| next | three |"
    );
    assert_eq!(result.row_count, 2);
}

#[test]
fn borderless_lines_are_separate_rows_even_with_multiline() {
    let input = "│ A │ B │\n│ 1 │ 2 │\n│ 3 │ 4 │";
    for row_mode in [RowMode::Auto, RowMode::SingleLine, RowMode::Multiline] {
        let result = convert_with_options(
            input,
            Options {
                row_mode,
                ..Options::default()
            },
        );
        assert_eq!(
            result.markdown,
            "| A | B |\n| --- | --- |\n| 1 | 2 |\n| 3 | 4 |"
        );
    }
}

#[test]
fn preserves_empty_edge_columns_and_empty_rows() {
    let input = "+---+---+---+\n|| Header ||\n+---+---+---+\n|| Value ||\n+---+---+---+\n||||\n+---+---+---+";
    let result = convert(input);
    assert_eq!(
        result.markdown,
        "|  | Header |  |\n| --- | --- | --- |\n|  | Value |  |\n|  |  |  |"
    );
    assert_eq!(result.column_count, 3);
    assert_eq!(result.row_count, 2);
}

#[test]
fn escapes_literal_pipes_in_unicode_cells() {
    let input = "│ Name │ Regex │\n│ A │ x | y │";
    assert_eq!(
        convert(input).markdown,
        "| Name | Regex |\n| --- | --- |\n| A | x \\| y |"
    );
}

#[test]
fn preserves_escaped_pipes_and_does_not_treat_them_as_columns() {
    let input = r"| Name | Regex |
| A | x \| y |";
    assert_eq!(
        convert(input).markdown,
        "| Name | Regex |\n| --- | --- |\n| A | x \\| y |"
    );
    let unicode = "│ Name │ Regex │\n│ A │ x \\| y │";
    assert_eq!(convert(unicode).markdown, convert(input).markdown);
}

#[test]
fn escapes_pipes_after_even_number_of_backslashes() {
    let input = "│ Header │\n│ x \\\\| y │";
    assert_eq!(
        convert(input).markdown,
        "| Header |\n| --- |\n| x \\\\\\| y |"
    );
}

#[test]
fn pads_ragged_rows_and_reports_warning() {
    let result = convert("│ A │ B │\n│ 1 │\n│ 2 │ 3 │ 4 │");
    assert_eq!(
        result.markdown,
        "| A | B |  |\n| --- | --- | --- |\n| 1 |  |  |\n| 2 | 3 | 4 |"
    );
    assert_eq!(result.column_count, 3);
    assert_eq!(result.warnings.len(), 2);
    assert_eq!(result.warnings[0], "Row 1 has 2 columns; padded to 3.");
    assert_eq!(result.warnings[1], "Row 2 has 1 columns; padded to 3.");
}

#[test]
fn preserves_surrounding_text_whitespace_and_newlines() {
    let input = "\n# Title  \n\n\n│ A │ B │\n│ 1 │ 2 │\n\nText with trailing spaces  \n\n";
    assert_eq!(
        convert(input).markdown,
        "\n# Title  \n\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n\nText with trailing spaces  \n\n"
    );
}

#[test]
fn counts_multiple_tables_of_different_widths() {
    let result = convert("│ A │\n│ 1 │\n\nSecond:\n│ X │ Y │\n│ 2 │ 3 │");
    assert_eq!(result.table_count, 2);
    assert_eq!(result.column_count, 2);
    assert_eq!(result.row_count, 2);
    assert_eq!(
        result.markdown,
        "| A |\n| --- |\n| 1 |\n\nSecond:\n| X | Y |\n| --- | --- |\n| 2 | 3 |"
    );
}

#[test]
fn separates_adjacent_boxed_tables() {
    let input = "┌───┐\n│ A │\n│ 1 │\n└───┘\n┌───┐\n│ B │\n│ 2 │\n└───┘";
    let result = convert(input);
    assert_eq!(result.table_count, 2);
    assert_eq!(
        result.markdown,
        "| A |\n| --- |\n| 1 |\n\n| B |\n| --- |\n| 2 |"
    );
}

#[test]
fn leaves_existing_markdown_tables_unchanged() {
    let input = "| Name | Value |\n| :--- | ---: |\n| A | 1 |\n";
    let result = convert(input);
    assert_eq!(result.markdown, input);
    assert_eq!(result.table_count, 0);
}

#[test]
fn conversion_is_idempotent() {
    let first = convert("│ Name │ Value │\n│ A │ 1 │");
    assert_eq!(convert(&first.markdown).markdown, first.markdown);
}

#[test]
fn leaves_plain_text_and_rule_only_blocks_unchanged() {
    for input in ["Some text  \n\n", "┌───┐\n└───┘\n", "---\n", "+---+\n"] {
        let result = convert(input);
        assert_eq!(result.markdown, input);
        assert_eq!(result.table_count, 0);
        assert_eq!(result.warnings.len(), 1);
    }
}

#[test]
fn empty_input_does_not_warn() {
    for input in ["", "\n", "  \n\n"] {
        let result = convert(input);
        assert_eq!(result.markdown, input);
        assert!(result.warnings.is_empty());
    }
}

#[test]
fn accepts_crlf_and_single_column_tables() {
    let result = convert("┌───┐\r\n│ H │\r\n├───┤\r\n│ V │\r\n└───┘\r\n");
    assert_eq!(result.markdown, "| H |\n| --- |\n| V |\n");
    assert_eq!(result.column_count, 1);
}

#[test]
fn header_only_table_warns() {
    let result = convert("│ A │ B │");
    assert_eq!(result.markdown, "| A | B |\n| --- | --- |");
    assert_eq!(result.row_count, 0);
    assert_eq!(result.warnings, ["Only a header row was found."]);
}

#[test]
fn dash_only_unicode_cells_are_not_horizontal_rules() {
    let input = "┌───┐\n│ Header │\n├───┤\n│ - │\n│ = │\n│ ─ │\n│ --- │\n└───┘";
    let result = convert(input);
    assert_eq!(
        result.markdown,
        "| Header |\n| --- |\n| - |\n| = |\n| ─ |\n| --- |"
    );
    assert_eq!(result.row_count, 4);
}

#[test]
fn dash_only_ascii_data_does_not_disable_box_conversion() {
    let result = convert("+-----+\n| Header |\n+-----+\n| --- |\n+-----+");
    assert_eq!(result.markdown, "| Header |\n| --- |\n| --- |");
    assert_eq!(result.table_count, 1);
}

#[test]
fn converts_box_after_existing_markdown_without_blank_line() {
    let input = "| A |\n| --- |\n| 1 |\n┌───┐\n│ B │\n│ 2 │\n└───┘";
    let result = convert(input);
    assert_eq!(
        result.markdown,
        "| A |\n| --- |\n| 1 |\n| B |\n| --- |\n| 2 |"
    );
    assert_eq!(result.table_count, 1);
}

#[test]
fn converts_mixed_vertical_border_styles() {
    let input = "╔═══╤═══╗\n║ A │ B ║\n╟───┼───╢\n║ 1 │ 2 ║\n╚═══╧═══╝";
    assert_eq!(
        convert(input).markdown,
        "| A | B |\n| --- | --- |\n| 1 | 2 |"
    );
}

#[test]
fn single_line_mode_preserves_empty_first_cell_rows() {
    let input = "+---+---+\n| A | B |\n+---+---+\n| 1 | 2 |\n|   | 3 |\n|   |   |\n+---+---+";
    let result = convert_with_options(
        input,
        Options {
            row_mode: RowMode::SingleLine,
            ..Options::default()
        },
    );
    assert_eq!(
        result.markdown,
        "| A | B |\n| --- | --- |\n| 1 | 2 |\n|  | 3 |\n|  |  |"
    );
    assert_eq!(result.row_count, 3);
}

#[test]
fn multiline_mode_preserves_blank_fragments_and_cell_alignment() {
    let input = "┌───┬───┐\n│ A │ B │\n├───┼───┤\n│ a │ 1 │\n│   │ 2 │\n│ b │   │\n└───┴───┘";
    let result = convert_with_options(
        input,
        Options {
            row_mode: RowMode::Multiline,
            ..Options::default()
        },
    );
    assert_eq!(
        result.markdown,
        "| A | B |\n| --- | --- |\n| a<br><br>b | 1<br>2<br> |"
    );
}

#[test]
fn prose_ending_in_pipe_does_not_cause_extra_blank_lines() {
    let result = convert("│ A │\n│ 1 │\nprose|\n│ B │\n│ 2 │");
    assert_eq!(
        result.markdown,
        "| A |\n| --- |\n| 1 |\nprose|\n| B |\n| --- |\n| 2 |"
    );
}

#[test]
fn malformed_outer_delimiters_pass_through_unchanged() {
    for line in [
        "│ missing end",
        "missing start │",
        "│ mismatched ║",
        "┃ mismatched │",
        "| mismatched │",
        "│ mismatched |",
        "│ - ║",
        "┃ ─ │",
        "| missing end",
        "missing start |",
        "╎ unsupported ╎",
        "┆ unsupported ┆",
    ] {
        let input = format!("Before  \r\n{line}\r\nAfter  ");
        let result = convert(&input);
        assert_eq!(result.markdown, input, "{line:?}");
        assert_eq!(result.table_count, 0, "{line:?}");
        assert_eq!(result.warnings.len(), 1, "{line:?}");
    }
}

#[test]
fn unusual_rule_like_text_is_not_consumed_next_to_valid_rows() {
    for line in ["───", "━━━", "═══", "┄┄┄", "╱──╲", "│ - ║", "┌─oops─┐"]
    {
        let input = format!("{line}\n│ Header │\n│ Value │\n{line}");
        let result = convert(&input);
        assert_eq!(
            result.markdown,
            format!("{line}\n| Header |\n| --- |\n| Value |\n{line}"),
            "{line:?}"
        );
        assert_eq!(result.table_count, 1, "{line:?}");
        assert_eq!(result.row_count, 1, "{line:?}");
        assert!(result.warnings.is_empty(), "{line:?}");
    }
}
