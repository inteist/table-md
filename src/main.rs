use std::env;
use std::io::{self, IsTerminal, Read, Write};
use std::process::ExitCode;

use table_md::{Options, RowMode, convert_with_options};

const HELP: &str = "table-md — convert Unicode and ASCII box tables to Markdown

Usage: table-md [OPTIONS]

Reads UTF-8 text from stdin and writes converted Markdown to stdout.
By default, blank-first-cell lines in bordered tables continue the previous row.
Use --single-line for empty-first-cell rows, or --multiline for wrapped cells.
When entering text directly, finish input with Ctrl-D (Unix) or Ctrl-Z (Windows).

Options:
  --no-header   Generate Column 1, Column 2, … instead of using the first row
  --multiline   Join all cell lines between horizontal rules with <br>
  --single-line  Treat every cell line as a separate row
  -q, --quiet  Suppress conversion warnings
  -h, --help   Print help
  -V, --version  Print version

Examples:
  table-md < table.txt
  pbpaste | table-md | pbcopy
  table-md --no-header < table.txt
";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "table-md: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<()> {
    let mut options = Options::default();
    let mut quiet = false;

    for argument in env::args_os().skip(1) {
        match argument.to_str() {
            Some("--no-header") => options.first_row_is_header = false,
            Some("--multiline") => options.row_mode = RowMode::Multiline,
            Some("--single-line") => options.row_mode = RowMode::SingleLine,
            Some("-q" | "--quiet") => quiet = true,
            Some("-h" | "--help") => {
                return io::stdout().lock().write_all(HELP.as_bytes());
            }
            Some("-V" | "--version") => {
                return writeln!(
                    io::stdout().lock(),
                    "table-md {}",
                    env!("CARGO_PKG_VERSION")
                );
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown argument {argument:?}; use --help for usage"),
                ));
            }
        }
    }

    let stdin = io::stdin();
    if stdin.is_terminal() && !quiet {
        let _ = writeln!(
            io::stderr().lock(),
            "Paste a table, then finish input with Ctrl-D (Unix) or Ctrl-Z (Windows)."
        );
    }
    let mut input = String::new();
    stdin.lock().read_to_string(&mut input)?;
    let result = convert_with_options(&input, options);

    if !quiet {
        for warning in &result.warnings {
            let _ = writeln!(io::stderr().lock(), "table-md: warning: {warning}");
        }
    }

    let mut stdout = io::stdout().lock();
    stdout.write_all(result.markdown.as_bytes())?;
    if result.table_count > 0 && !result.markdown.ends_with('\n') {
        stdout.write_all(b"\n")?;
    }
    stdout.flush()
}
