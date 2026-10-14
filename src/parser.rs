#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RowMode {
    #[default]
    Auto,
    SingleLine,
    Multiline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    pub first_row_is_header: bool,
    pub row_mode: RowMode,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            first_row_is_header: true,
            row_mode: RowMode::Auto,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedTable {
    pub markdown: String,
    pub table_count: usize,
    pub row_count: usize,
    pub column_count: usize,
    pub warnings: Vec<String>,
}

pub fn convert(input: &str) -> ParsedTable {
    convert_with_options(input, Options::default())
}

pub fn parse_ascii_table(input: &str, first_row_is_header: bool) -> ParsedTable {
    convert_with_options(
        input,
        Options {
            first_row_is_header,
            ..Options::default()
        },
    )
}

pub fn convert_with_options(input: &str, options: Options) -> ParsedTable {
    let mut result = ParsedTable {
        markdown: String::new(),
        table_count: 0,
        row_count: 0,
        column_count: 0,
        warnings: Vec::new(),
    };
    let mut block = Vec::new();
    let mut previous_was_table = false;

    for line in input.split_inclusive('\n') {
        let trimmed = line.trim();
        if is_horizontal_rule(trimmed) || detect_cell_delimiter(trimmed).is_some() {
            if is_horizontal_rule(trimmed)
                && ((block
                    .last()
                    .is_some_and(|line: &&str| is_horizontal_rule(line.trim()))
                    && block
                        .iter()
                        .any(|line| detect_cell_delimiter(line.trim()).is_some()))
                    || is_markdown_table(&block))
            {
                append_block(&mut result, &block, options, &mut previous_was_table);
                block.clear();
            }
            block.push(line);
        } else {
            append_block(&mut result, &block, options, &mut previous_was_table);
            block.clear();
            result.markdown.push_str(line);
            previous_was_table = false;
        }
    }
    append_block(&mut result, &block, options, &mut previous_was_table);

    if result.table_count == 0 && !input.trim().is_empty() {
        result
            .warnings
            .push("No box-table rows were found; input was left unchanged.".into());
    }
    result
}

fn append_block(
    result: &mut ParsedTable,
    block: &[&str],
    options: Options,
    previous_was_table: &mut bool,
) {
    if block.is_empty() {
        return;
    }
    let Some(table) = convert_table_block(block, options) else {
        for line in block {
            result.markdown.push_str(line);
        }
        *previous_was_table = false;
        return;
    };
    if *previous_was_table {
        result.markdown.push('\n');
    }
    result.markdown.push_str(&table.markdown);
    if block.last().is_some_and(|line| line.ends_with('\n')) {
        result.markdown.push('\n');
    }
    result.table_count += 1;
    result.row_count += table.row_count;
    result.column_count = result.column_count.max(table.column_count);
    result.warnings.extend(table.warnings);
    *previous_was_table = true;
}

fn convert_table_block(block: &[&str], options: Options) -> Option<ParsedTable> {
    if is_markdown_table(block) {
        return None;
    }
    let groups = collect_row_groups(block, options.row_mode);
    let mut rows: Vec<Vec<String>> = groups
        .into_iter()
        .map(|group| collapse_group(group, options.row_mode))
        .collect();
    let column_count = rows.iter().map(Vec::len).max()?;
    let mut warnings = Vec::new();

    for (index, row) in rows.iter_mut().enumerate() {
        if row.len() != column_count {
            warnings.push(format!(
                "Row {} has {} columns; padded to {column_count}.",
                index + 1,
                row.len()
            ));
            row.resize(column_count, String::new());
        }
    }
    let header = if options.first_row_is_header {
        rows.remove(0)
    } else {
        (1..=column_count)
            .map(|index| format!("Column {index}"))
            .collect()
    };
    if rows.is_empty() {
        warnings.push("Only a header row was found.".into());
    }
    Some(ParsedTable {
        markdown: render_markdown_table(&header, &rows),
        table_count: 1,
        row_count: rows.len(),
        column_count,
        warnings,
    })
}

fn collect_row_groups<'a>(block: &[&'a str], row_mode: RowMode) -> Vec<Vec<&'a str>> {
    let has_rules = block.iter().any(|line| is_horizontal_rule(line.trim()));
    let mut groups = Vec::new();
    let mut current = Vec::new();

    for line in block {
        let trimmed = line.trim();
        if is_horizontal_rule(trimmed) {
            if !current.is_empty() {
                groups.push(std::mem::take(&mut current));
            }
        } else if let Some(delimiter) = detect_cell_delimiter(trimmed) {
            let first_cell_empty = split_cells(trimmed, delimiter)[0].trim().is_empty();
            let starts_row = !has_rules
                || match row_mode {
                    RowMode::Auto => !first_cell_empty,
                    RowMode::SingleLine => true,
                    RowMode::Multiline => false,
                };
            if !current.is_empty() && starts_row {
                groups.push(std::mem::take(&mut current));
            }
            current.push(trimmed);
        }
    }
    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

fn is_horizontal_rule(line: &str) -> bool {
    if detect_cell_delimiter(line).is_some() {
        return false;
    }
    let mut has_corner = false;
    let mut has_rule = false;
    for ch in line.chars().filter(|ch| !ch.is_whitespace()) {
        match ch {
            '+' => has_corner = true,
            '-' | '=' => has_rule = true,
            ':' => {}
            '─' | '━' | '═' | '╌' | '╍' | '┄' | '┅' | '┈' | '┉' => {
                has_rule = true
            }
            '\u{250c}'..='\u{254b}' | '\u{2552}'..='\u{256c}' | '\u{256d}'..='\u{2570}' => {
                has_corner = true;
            }
            _ => return false,
        }
    }
    has_corner && has_rule
}

fn detect_cell_delimiter(line: &str) -> Option<char> {
    ['|', '│', '┃', '║'].into_iter().find(|delimiter| {
        line.starts_with(*delimiter) && line.ends_with(*delimiter) && line.chars().count() >= 2
    })
}

fn split_cells(line: &str, delimiter: char) -> Vec<&str> {
    let inner = &line[delimiter.len_utf8()..line.len() - delimiter.len_utf8()];
    let mut cells = Vec::new();
    let mut start = 0;
    let mut backslashes = 0;
    for (index, ch) in inner.char_indices() {
        let is_delimiter = if delimiter == '|' {
            ch == '|' && backslashes % 2 == 0
        } else {
            matches!(ch, '│' | '┃' | '║')
        };
        if is_delimiter {
            cells.push(&inner[start..index]);
            start = index + ch.len_utf8();
        }
        backslashes = if ch == '\\' { backslashes + 1 } else { 0 };
    }
    cells.push(&inner[start..]);
    cells
}

fn is_markdown_table(block: &[&str]) -> bool {
    block
        .first()
        .is_some_and(|line| detect_cell_delimiter(line.trim()) == Some('|'))
        && block
            .get(1)
            .is_some_and(|line| is_markdown_separator(line.trim()))
        && !block.iter().any(|line| is_horizontal_rule(line.trim()))
}

fn is_markdown_separator(line: &str) -> bool {
    if detect_cell_delimiter(line) != Some('|') {
        return false;
    }
    split_cells(line, '|').iter().all(|cell| {
        let rule = cell.trim().trim_matches(':');
        rule.len() >= 3 && rule.chars().all(|ch| ch == '-')
    })
}

fn collapse_group(lines: Vec<&str>, row_mode: RowMode) -> Vec<String> {
    let parsed: Vec<Vec<&str>> = lines
        .into_iter()
        .map(|line| split_cells(line, detect_cell_delimiter(line).unwrap()))
        .collect();
    let column_count = parsed.iter().map(Vec::len).max().unwrap_or(0);
    (0..column_count)
        .map(|column| {
            parsed
                .iter()
                .map(|line| line.get(column).copied().unwrap_or("").trim())
                .filter(|cell| row_mode == RowMode::Multiline || !cell.is_empty())
                .map(escape_markdown_cell)
                .collect::<Vec<_>>()
                .join("<br>")
        })
        .collect()
}

fn escape_markdown_cell(cell: &str) -> String {
    let mut escaped = String::new();
    let mut backslashes = 0;
    for ch in cell.chars() {
        if ch == '|' && backslashes % 2 == 0 {
            escaped.push('\\');
        }
        escaped.push(ch);
        backslashes = if ch == '\\' { backslashes + 1 } else { 0 };
    }
    escaped
}

fn render_markdown_table(header: &[String], body: &[Vec<String>]) -> String {
    let mut lines = Vec::with_capacity(body.len() + 2);
    lines.push(render_markdown_row(header));
    lines.push(render_markdown_row(&vec!["---".to_string(); header.len()]));
    lines.extend(body.iter().map(|row| render_markdown_row(row)));
    lines.join("\n")
}

fn render_markdown_row(cells: &[String]) -> String {
    format!("| {} |", cells.join(" | "))
}
