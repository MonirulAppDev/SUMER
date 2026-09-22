//! Integration tests for the `sumer` CLI.

use std::fs;
use sumer_cli::run_cli;

#[test]
fn test_cli_help_succeeds() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run_cli(&["--help".to_string()], &mut stdout, &mut stderr);

    assert_eq!(code, 0);
    let out = String::from_utf8_lossy(&stdout);
    assert!(out.contains("SUMER compiler"));
    assert!(out.contains("Commands:"));
    assert!(out.contains("parse"));
}

#[test]
fn test_cli_parse_help_succeeds() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run_cli(
        &["parse".to_string(), "--help".to_string()],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(code, 0);
    let out = String::from_utf8_lossy(&stdout);
    assert!(out.contains("Parse a SUMER source file and print its AST"));
    assert!(out.contains("Usage:"));
    assert!(out.contains("--spans"));
}

#[test]
fn test_cli_parse_valid_hello_file() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
    let file = workspace_root.join("examples/hello.sm");

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run_cli(
        &["parse".to_string(), file.to_string_lossy().to_string()],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(code, 0);
    let out = String::from_utf8_lossy(&stdout);
    assert!(out.contains("Program"));
    assert!(out.contains("Function: main"));
    assert!(out.contains("Call"));
    assert!(out.contains("Identifier: print"));
    assert!(out.contains("String: \"Hello, SUMER!\""));
}

#[test]
fn test_cli_parse_features_file() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
    let file = workspace_root.join("examples/features.sm");

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run_cli(
        &["parse".to_string(), file.to_string_lossy().to_string()],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(code, 0);
    let out = String::from_utf8_lossy(&stdout);
    assert!(out.contains("Struct: User"));
    assert!(out.contains("Enum: Status"));
    assert!(out.contains("Trait: Printable"));
    assert!(out.contains("Function: main"));
}

#[test]
fn test_cli_parse_with_spans_flag() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
    let file = workspace_root.join("examples/hello.sm");

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run_cli(
        &[
            "parse".to_string(),
            "--spans".to_string(),
            file.to_string_lossy().to_string(),
        ],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(code, 0);
    let out = String::from_utf8_lossy(&stdout);
    assert!(out.contains("Program [0..41]"));
    assert!(out.contains("Function: main [0..40]"));
}

#[test]
fn test_cli_parse_missing_file_fails_cleanly() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run_cli(
        &["parse".to_string(), "does-not-exist.sm".to_string()],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(code, 1);
    let err = String::from_utf8_lossy(&stderr);
    assert!(err.contains("error: could not read file 'does-not-exist.sm'"));
}

#[test]
fn test_cli_parse_invalid_file_reports_diagnostic() {
    let tmp_dir = std::env::temp_dir();
    let invalid_file = tmp_dir.join("sumer_test_invalid.sm");
    fs::write(&invalid_file, "fn main( {\n").expect("write temp file");

    let path_str = invalid_file.to_string_lossy().to_string();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run_cli(&["parse".to_string(), path_str], &mut stdout, &mut stderr);

    let _ = fs::remove_file(&invalid_file);

    assert_eq!(code, 1);
    let err = String::from_utf8_lossy(&stderr);
    assert!(err.contains("error:"));
    assert!(err.contains("1 | fn main( {"));
    assert!(err.contains('^'));
}

#[test]
fn test_cli_unknown_command() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run_cli(&["unknown_cmd".to_string()], &mut stdout, &mut stderr);

    assert_eq!(code, 1);
    let err = String::from_utf8_lossy(&stderr);
    assert!(err.contains("error: unknown command 'unknown_cmd'"));
}

#[test]
fn test_cli_missing_arg_to_parse() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run_cli(&["parse".to_string()], &mut stdout, &mut stderr);

    assert_eq!(code, 1);
    let err = String::from_utf8_lossy(&stderr);
    assert!(err.contains("error: 'sumer parse' requires a <FILE> argument"));
}

#[test]
fn test_cli_check_help_succeeds() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run_cli(
        &["check".to_string(), "--help".to_string()],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(code, 0);
    let out = String::from_utf8_lossy(&stdout);
    assert!(out.contains("Check a SUMER source file for syntax and semantic errors"));
    assert!(out.contains("Usage:"));
}

#[test]
fn test_cli_check_valid_hello_file() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
    let file = workspace_root.join("examples/hello.sm");

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run_cli(
        &["check".to_string(), file.to_string_lossy().to_string()],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(code, 0);
    assert!(stderr.is_empty());
}

#[test]
fn test_cli_check_valid_features_file() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
    let file = workspace_root.join("examples/features.sm");

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run_cli(
        &["check".to_string(), file.to_string_lossy().to_string()],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(code, 0);
    assert!(stderr.is_empty());
}

#[test]
fn test_cli_check_missing_arg() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run_cli(&["check".to_string()], &mut stdout, &mut stderr);

    assert_eq!(code, 1);
    let err = String::from_utf8_lossy(&stderr);
    assert!(err.contains("error: 'sumer check' requires a <FILE> argument"));
}

#[test]
fn test_cli_check_semantic_error_reports_diagnostic() {
    let tmp_dir = std::env::temp_dir();
    let invalid_file = tmp_dir.join("sumer_test_sema_err.sm");
    fs::write(
        &invalid_file,
        "fn main() {\n    print(undeclared_value)\n}\n",
    )
    .expect("write temp file");

    let path_str = invalid_file.to_string_lossy().to_string();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run_cli(&["check".to_string(), path_str], &mut stdout, &mut stderr);

    let _ = fs::remove_file(&invalid_file);

    assert_eq!(code, 1);
    let err = String::from_utf8_lossy(&stderr);
    assert!(err.contains("error: cannot find `undeclared_value` in this scope"));
    assert!(err.contains("2 |     print(undeclared_value)"));
    assert!(err.contains('^'));
}

#[test]
fn test_cli_check_type_error_reports_diagnostic() {
    let tmp_dir = std::env::temp_dir();
    let invalid_file = tmp_dir.join("sumer_test_type_err.sm");
    fs::write(
        &invalid_file,
        "fn main() {\n    let age: Int = \"Monir\"\n}\n",
    )
    .expect("write temp file");

    let path_str = invalid_file.to_string_lossy().to_string();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run_cli(&["check".to_string(), path_str], &mut stdout, &mut stderr);

    let _ = fs::remove_file(&invalid_file);

    assert_eq!(code, 1);
    let err = String::from_utf8_lossy(&stderr);
    assert!(err.contains("error: type mismatch"));
    assert!(err.contains("expected `Int`, found `String`"));
    assert!(err.contains("2 |     let age: Int = \"Monir\""));
    assert!(err.contains('^'));
}
