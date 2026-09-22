//! Command Line Interface implementation for the SUMER compiler.

use std::fs;
use std::io::Write;
use std::path::Path;

use sumer_ast::AstPrinter;
use sumer_driver::parse_source;
use sumer_span::SourceMap;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const TOP_LEVEL_HELP: &str = "\
SUMER compiler

Usage:
    sumer <COMMAND>

Commands:
    parse    Parse a SUMER source file and print its AST
    check    Check a SUMER source file for syntax and semantic errors
    help     Print this message or the help of the given subcommand(s)

Options:
    -h, --help       Print help
    -V, --version    Print version
";

const PARSE_HELP: &str = "\
Parse a SUMER source file and print its AST

Usage:
    sumer parse [OPTIONS] <FILE>

Arguments:
    <FILE>    Path to the SUMER source file (.sm)

Options:
    --spans          Include source spans in AST output
    -h, --help       Print help
";

const CHECK_HELP: &str = "\
Check a SUMER source file for syntax and semantic errors

Usage:
    sumer check [OPTIONS] <FILE>

Arguments:
    <FILE>    Path to the SUMER source file (.sm)

Options:
    -h, --help       Print help
";

/// Runs the SUMER CLI with the given command-line arguments.
///
/// Streams standard output to `stdout` and diagnostic/error output to `stderr`.
/// Returns the process exit code (`0` for success, non-zero for failure).
pub fn run_cli<W1: Write, W2: Write>(args: &[String], stdout: &mut W1, stderr: &mut W2) -> i32 {
    if args.is_empty() {
        let _ = write!(stdout, "{TOP_LEVEL_HELP}");
        return 0;
    }

    match args[0].as_str() {
        "-h" | "--help" => {
            let _ = write!(stdout, "{TOP_LEVEL_HELP}");
            0
        }
        "-V" | "--version" => {
            let _ = writeln!(stdout, "sumer {VERSION}");
            0
        }
        "help" => {
            if args.len() > 1 && args[1] == "parse" {
                let _ = write!(stdout, "{PARSE_HELP}");
            } else if args.len() > 1 && args[1] == "check" {
                let _ = write!(stdout, "{CHECK_HELP}");
            } else {
                let _ = write!(stdout, "{TOP_LEVEL_HELP}");
            }
            0
        }
        "parse" => handle_parse_command(&args[1..], stdout, stderr),
        "check" => handle_check_command(&args[1..], stdout, stderr),
        unknown if unknown.starts_with('-') => {
            let _ = writeln!(
                stderr,
                "error: unknown flag '{unknown}'. Run 'sumer --help' for usage."
            );
            1
        }
        unknown => {
            let _ = writeln!(
                stderr,
                "error: unknown command '{unknown}'. Run 'sumer --help' for usage."
            );
            1
        }
    }
}

fn handle_parse_command<W1: Write, W2: Write>(
    args: &[String],
    stdout: &mut W1,
    stderr: &mut W2,
) -> i32 {
    let mut show_spans = false;
    let mut file_path: Option<&str> = None;

    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => {
                let _ = write!(stdout, "{PARSE_HELP}");
                return 0;
            }
            "--spans" => {
                show_spans = true;
            }
            flag if flag.starts_with('-') => {
                let _ = writeln!(
                    stderr,
                    "error: unknown flag '{flag}'. Run 'sumer parse --help' for usage."
                );
                return 1;
            }
            path => {
                if file_path.is_some() {
                    let _ = writeln!(
                        stderr,
                        "error: unexpected argument '{path}'. 'sumer parse' accepts exactly one <FILE>."
                    );
                    return 1;
                }
                file_path = Some(path);
            }
        }
    }

    let Some(path_str) = file_path else {
        let _ = writeln!(
            stderr,
            "error: 'sumer parse' requires a <FILE> argument. Run 'sumer parse --help' for usage."
        );
        return 1;
    };

    let path = Path::new(path_str);
    let source_content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            let _ = writeln!(stderr, "error: could not read file '{path_str}': {err}");
            return 1;
        }
    };

    let mut source_map = SourceMap::new();
    let source_id = source_map.add(path_str, &source_content);

    match parse_source(source_id, &source_content) {
        Ok(program) => {
            let output = AstPrinter::new().with_spans(show_spans).print(&program);
            let _ = write!(stdout, "{output}");
            0
        }
        Err(err) => {
            let diag = err.format_diagnostic(&source_map);
            let _ = write!(stderr, "{diag}");
            1
        }
    }
}

fn handle_check_command<W1: Write, W2: Write>(
    args: &[String],
    stdout: &mut W1,
    stderr: &mut W2,
) -> i32 {
    let mut file_path: Option<&str> = None;

    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => {
                let _ = write!(stdout, "{CHECK_HELP}");
                return 0;
            }
            flag if flag.starts_with('-') => {
                let _ = writeln!(
                    stderr,
                    "error: unknown flag '{flag}'. Run 'sumer check --help' for usage."
                );
                return 1;
            }
            path => {
                if file_path.is_some() {
                    let _ = writeln!(
                        stderr,
                        "error: unexpected argument '{path}'. 'sumer check' accepts exactly one <FILE>."
                    );
                    return 1;
                }
                file_path = Some(path);
            }
        }
    }

    let Some(path_str) = file_path else {
        let _ = writeln!(
            stderr,
            "error: 'sumer check' requires a <FILE> argument. Run 'sumer check --help' for usage."
        );
        return 1;
    };

    let path = Path::new(path_str);
    let source_content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            let _ = writeln!(stderr, "error: could not read file '{path_str}': {err}");
            return 1;
        }
    };

    let mut source_map = SourceMap::new();
    let source_id = source_map.add(path_str, &source_content);

    match sumer_driver::check_source(source_id, &source_content) {
        Ok(result) => {
            if result.has_errors() {
                let diag = sumer_diagnostics::render_all(&result.diagnostics, &source_map);
                let _ = write!(stderr, "{diag}");
                1
            } else {
                0
            }
        }
        Err(err) => {
            let diag = err.format_diagnostic(&source_map);
            let _ = write!(stderr, "{diag}");
            1
        }
    }
}
