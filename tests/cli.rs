#![cfg(feature = "cli")]

use std::io::Write;
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const ASCII_TABLE: &str =
    "+-------+-------+\n| Name  | Score |\n+-------+-------+\n| Alice | 42    |\n+-------+-------+";
const ASCII_MARKDOWN: &str = "| Name | Score |\n| --- | --- |\n| Alice | 42 |\n";
const RAGGED_TABLE: &str = "| Name | Score |\n| Alice |\n";
const RAGGED_MARKDOWN: &str = "| Name | Score |\n| --- | --- |\n| Alice |  |\n";
const RAGGED_WARNING: &str = "table-md: warning: Row 2 has 1 columns; padded to 2.\n";

fn spawn(args: &[&str]) -> Child {
    Command::new(env!("CARGO_BIN_EXE_table-md"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn table-md")
}

fn wait(mut child: Child) -> Output {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if child.try_wait().expect("failed to poll table-md").is_some() {
            return child.wait_with_output().expect("failed to collect output");
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().expect("failed to reap table-md");
            panic!("table-md did not exit within five seconds: {output:?}");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn run(args: &[&str], input: &[u8]) -> Output {
    let mut child = spawn(args);
    child
        .stdin
        .take()
        .expect("missing stdin pipe")
        .write_all(input)
        .expect("failed to write stdin");
    wait(child)
}

fn run_with_open_stdin(args: &[&str]) -> Output {
    let mut child = spawn(args);
    let stdin = child.stdin.take().expect("missing stdin pipe");
    let output = wait(child);
    drop(stdin);
    output
}

fn assert_success(output: &Output, stdout: &str, stderr: &str) {
    assert!(output.status.success(), "unexpected failure: {output:?}");
    assert_eq!(output.stdout, stdout.as_bytes());
    assert_eq!(output.stderr, stderr.as_bytes());
}

#[test]
fn converts_ascii_stdin_and_terminates_output_with_one_newline() {
    for input in [ASCII_TABLE.to_owned(), format!("{ASCII_TABLE}\n")] {
        let output = run(&[], input.as_bytes());
        assert_success(&output, ASCII_MARKDOWN, "");
    }
}

#[test]
fn converts_unicode_stdin_and_preserves_cell_text() {
    let input = "┌──────┬───────┐\n│ 名字 │ Café  │\n├──────┼───────┤\n│ 李雷 │ naïve │\n└──────┴───────┘\n";
    let output = run(&[], input.as_bytes());
    assert_success(
        &output,
        "| 名字 | Café |\n| --- | --- |\n| 李雷 | naïve |\n",
        "",
    );
}

#[test]
fn no_header_generates_names_and_keeps_the_first_row() {
    let output = run(&["--no-header"], ASCII_TABLE.as_bytes());
    assert_success(
        &output,
        "| Column 1 | Column 2 |\n| --- | --- |\n| Name | Score |\n| Alice | 42 |\n",
        "",
    );
}

#[test]
fn multiline_joins_cell_lines_between_rules() {
    let input = "+-------+-------+\n| Name  | Note  |\n+-------+-------+\n| Alice | first |\n| Bob   | next  |\n+-------+-------+\n";
    let default_output = run(&[], input.as_bytes());
    assert_success(
        &default_output,
        "| Name | Note |\n| --- | --- |\n| Alice | first |\n| Bob | next |\n",
        "",
    );
    let output = run(&["--multiline"], input.as_bytes());
    assert_success(
        &output,
        "| Name | Note |\n| --- | --- |\n| Alice<br>Bob | first<br>next |\n",
        "",
    );
}

#[test]
fn conversion_warnings_go_to_stderr_not_stdout() {
    let output = run(&[], RAGGED_TABLE.as_bytes());
    assert_success(&output, RAGGED_MARKDOWN, RAGGED_WARNING);
}

#[test]
fn quiet_suppresses_warnings_without_changing_stdout() {
    for flag in ["--quiet", "-q"] {
        let output = run(&[flag], RAGGED_TABLE.as_bytes());
        assert_success(&output, RAGGED_MARKDOWN, "");
    }
}

#[test]
fn help_exits_without_waiting_for_stdin_eof() {
    for flag in ["--help", "-h"] {
        let output = run_with_open_stdin(&[flag]);
        assert!(output.status.success(), "unexpected failure: {output:?}");
        assert!(output.stderr.is_empty(), "unexpected stderr: {output:?}");
        let stdout = String::from_utf8(output.stdout).expect("help was not UTF-8");
        assert!(
            stdout.starts_with("table-md — convert Unicode and ASCII box tables to Markdown\n")
        );
        assert!(stdout.contains("Usage: table-md [OPTIONS]"));
        for option in [
            "--no-header",
            "--multiline",
            "--single-line",
            "--quiet",
            "--help",
            "--version",
        ] {
            assert!(stdout.contains(option), "help is missing {option}");
        }
    }
}

#[test]
fn version_exits_without_waiting_for_stdin_eof() {
    for flag in ["--version", "-V"] {
        let output = run_with_open_stdin(&[flag]);
        assert_success(
            &output,
            concat!("table-md ", env!("CARGO_PKG_VERSION"), "\n"),
            "",
        );
    }
}

#[test]
fn bad_arguments_fail_without_waiting_for_stdin_eof() {
    for argument in ["--unknown", "input.txt"] {
        let output = run_with_open_stdin(&[argument]);
        assert!(!output.status.success(), "unexpected success: {output:?}");
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty(), "unexpected stdout: {output:?}");
        assert_eq!(
            output.stderr,
            format!("table-md: unknown argument {argument:?}; use --help for usage\n").as_bytes(),
        );
    }
}

#[test]
fn empty_input_succeeds_without_output_or_warnings() {
    let output = run(&[], b"");
    assert_success(&output, "", "");
}

#[test]
fn plain_prose_passes_through_unchanged_with_a_warning() {
    for input in [
        "Plain prose, not a table.",
        "Café prose.\n\nAnother line.\n",
    ] {
        let output = run(&[], input.as_bytes());
        assert_success(
            &output,
            input,
            "table-md: warning: No box-table rows were found; input was left unchanged.\n",
        );
        let quiet_output = run(&["--quiet"], input.as_bytes());
        assert_success(&quiet_output, input, "");
    }
}

#[test]
fn invalid_utf8_fails_without_stdout_even_when_quiet() {
    for args in [&[][..], &["--quiet"][..]] {
        let output = run(args, b"valid prefix\n\xff\xfe\n");
        assert!(!output.status.success(), "unexpected success: {output:?}");
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty(), "unexpected stdout: {output:?}");
        let stderr = String::from_utf8(output.stderr).expect("error was not UTF-8");
        assert!(
            stderr.starts_with("table-md: "),
            "unexpected error: {stderr}"
        );
        assert!(stderr.contains("UTF-8"), "unexpected error: {stderr}");
        assert!(!stderr.contains("warning:"), "unexpected warning: {stderr}");
    }
}

#[test]
fn single_line_keeps_blank_first_column_rows_separate() {
    let input = "+---+---+\n| A | B |\n+---+---+\n| 1 | 2 |\n|   | 3 |\n+---+---+";
    let output = run(&["--single-line"], input.as_bytes());
    assert_success(
        &output,
        "| A | B |\n| --- | --- |\n| 1 | 2 |\n|  | 3 |\n",
        "",
    );
}

#[test]
fn closed_stderr_does_not_prevent_stdout_conversion() {
    let mut child = spawn(&[]);
    drop(child.stderr.take());
    child
        .stdin
        .take()
        .unwrap()
        .write_all(RAGGED_TABLE.as_bytes())
        .unwrap();
    let output = wait(child);
    assert_success(&output, RAGGED_MARKDOWN, "");
}

#[test]
fn closed_stdout_is_treated_as_successful_broken_pipe() {
    let mut child = spawn(&[]);
    drop(child.stdout.take());
    child
        .stdin
        .take()
        .unwrap()
        .write_all(ASCII_TABLE.as_bytes())
        .unwrap();
    let output = wait(child);
    assert_success(&output, "", "");
}
