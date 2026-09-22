//! Compiler pipeline orchestration and driver for SUMER.
//!
//! Provides high-level driver entry points for running the compiler frontend:
//! lexing, parsing, and diagnostic formatting.

use std::error::Error;
use std::fmt;

use sumer_ast::Program;
use sumer_lexer::{Lexer, LexerError};
use sumer_parser::{ParseError, parse};
use sumer_span::{SourceId, SourceMap, Span};

/// Errors encountered during driver execution of compiler phases.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DriverError {
    /// Lexical analysis failure.
    Lexer(LexerError),
    /// Syntax analysis / parsing failure.
    Parser(ParseError),
}

impl DriverError {
    /// Returns the source span where the error occurred.
    pub fn span(&self) -> Span {
        match self {
            Self::Lexer(e) => e.span,
            Self::Parser(e) => e.span,
        }
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        match self {
            Self::Lexer(e) => &e.message,
            Self::Parser(e) => &e.message,
        }
    }

    /// Converts this driver error into a structured [`sumer_diagnostics::Diagnostic`].
    pub fn to_diagnostic(&self) -> sumer_diagnostics::Diagnostic {
        match self {
            Self::Lexer(e) => e.to_diagnostic(),
            Self::Parser(e) => e.to_diagnostic(),
        }
    }

    /// Formats the error into a human-readable diagnostic message using [`sumer_diagnostics`].
    pub fn format_diagnostic(&self, source_map: &SourceMap) -> String {
        let diag = self.to_diagnostic();
        sumer_diagnostics::render(&diag, source_map)
    }
}

impl From<&DriverError> for sumer_diagnostics::Diagnostic {
    fn from(err: &DriverError) -> Self {
        err.to_diagnostic()
    }
}

impl From<DriverError> for sumer_diagnostics::Diagnostic {
    fn from(err: DriverError) -> Self {
        err.to_diagnostic()
    }
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lexer(e) => write!(f, "lexer error: {e}"),
            Self::Parser(e) => write!(f, "parse error: {e}"),
        }
    }
}

impl Error for DriverError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Lexer(e) => Some(e),
            Self::Parser(e) => Some(e),
        }
    }
}

/// Runs the lexer and parser pipeline on the provided source code, returning a [`Program`] AST.
///
/// # Errors
/// Returns [`DriverError::Lexer`] if tokenization fails, or [`DriverError::Parser`]
/// if syntax analysis fails.
pub fn parse_source(source_id: SourceId, source: &str) -> Result<Program, DriverError> {
    let tokens = Lexer::new(source_id, source)
        .lex()
        .map_err(DriverError::Lexer)?;
    parse(&tokens).map_err(DriverError::Parser)
}

/// Runs lexing, parsing, and semantic analysis on the provided source code,
/// returning the resulting [`sumer_sema::SemanticResult`].
///
/// # Errors
/// Returns [`DriverError::Lexer`] if tokenization fails, or [`DriverError::Parser`]
/// if syntax analysis fails.
pub fn check_source(
    source_id: SourceId,
    source: &str,
) -> Result<sumer_sema::SemanticResult, DriverError> {
    let program = parse_source(source_id, source)?;
    Ok(sumer_sema::analyze(&program))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_driver_parse_source_success() {
        let mut source_map = SourceMap::new();
        let src = "fn main() { print(\"Hello, SUMER!\") }";
        let id = source_map.add("hello.sm", src);

        let result = parse_source(id, src);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn test_driver_parse_source_parse_error() {
        let mut source_map = SourceMap::new();
        let src = "fn main( {";
        let id = source_map.add("hello.sm", src);

        let result = parse_source(id, src);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let diag = err.format_diagnostic(&source_map);

        assert!(diag.contains("error:"));
        assert!(diag.contains("--> hello.sm:1:"));
        assert!(diag.contains("fn main( {"));
        assert!(diag.contains('^'));
    }

    #[test]
    fn test_driver_parse_source_lexer_error() {
        let mut source_map = SourceMap::new();
        let src = "let x = \"unclosed string";
        let id = source_map.add("test.sm", src);

        let result = parse_source(id, src);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let diag = err.format_diagnostic(&source_map);

        assert!(diag.contains("error:"));
        assert!(diag.contains("--> test.sm:1:"));
    }

    #[test]
    fn test_driver_check_source_success() {
        let mut source_map = SourceMap::new();
        let src = "fn main() { let x = 10; print(x) }";
        let id = source_map.add("check.sm", src);

        let result = check_source(id, src);
        assert!(result.is_ok());
        let sema = result.unwrap();
        assert!(!sema.has_errors());
    }

    #[test]
    fn test_driver_check_source_semantic_error() {
        let mut source_map = SourceMap::new();
        let src = "fn main() { print(undeclared_var) }";
        let id = source_map.add("err.sm", src);

        let result = check_source(id, src);
        assert!(result.is_ok());
        let sema = result.unwrap();
        assert!(sema.has_errors());
        let diag = sumer_diagnostics::render_all(&sema.diagnostics, &source_map);
        assert!(diag.contains("cannot find `undeclared_var` in this scope"));
    }
}
