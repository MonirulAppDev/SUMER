//! Parser error definitions and error reporting types.

use std::error::Error;
use std::fmt;

use sumer_lexer::Token;
use sumer_span::Span;

/// Category of parser errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// Unexpected token encountered where another token was expected.
    UnexpectedToken { expected: String, found: String },
    /// Unexpected EOF encountered.
    UnexpectedEof { expected: String },
    /// Expected an identifier.
    ExpectedIdentifier { found: String },
    /// Invalid declaration syntax.
    InvalidDeclaration(String),
    /// Invalid statement syntax.
    InvalidStatement(String),
    /// Invalid type syntax.
    InvalidType(String),
    /// Invalid attribute syntax.
    InvalidAttribute(String),
    /// Feature not yet supported in this parser phase.
    UnsupportedFeature(String),
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken { expected, found } => {
                write!(f, "expected {expected}, found {found}")
            }
            Self::UnexpectedEof { expected } => {
                write!(f, "unexpected end of file, expected {expected}")
            }
            Self::ExpectedIdentifier { found } => {
                write!(f, "expected identifier, found {found}")
            }
            Self::InvalidDeclaration(msg) => write!(f, "invalid declaration: {msg}"),
            Self::InvalidStatement(msg) => write!(f, "invalid statement: {msg}"),
            Self::InvalidType(msg) => write!(f, "invalid type: {msg}"),
            Self::InvalidAttribute(msg) => write!(f, "invalid attribute: {msg}"),
            Self::UnsupportedFeature(msg) => write!(f, "unsupported syntax: {msg}"),
        }
    }
}

/// A parser error recording the reason, message, and source location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    /// Category of the error.
    pub kind: ParseErrorKind,
    /// Human-readable error message.
    pub message: String,
    /// Span in the source file where the error occurred.
    pub span: Span,
}

impl ParseError {
    /// Creates a new `ParseError`.
    pub fn new(kind: ParseErrorKind, message: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
        }
    }

    /// Creates an unexpected token error.
    pub fn unexpected_token(expected: impl Into<String>, found: &Token) -> Self {
        let expected = expected.into();
        let found_str = format!("{}", found.kind());
        Self::new(
            ParseErrorKind::UnexpectedToken {
                expected: expected.clone(),
                found: found_str.clone(),
            },
            format!("expected {expected}, found {found_str}"),
            found.span(),
        )
    }

    /// Creates an unexpected EOF error.
    pub fn unexpected_eof(expected: impl Into<String>, span: Span) -> Self {
        let expected = expected.into();
        Self::new(
            ParseErrorKind::UnexpectedEof {
                expected: expected.clone(),
            },
            format!("unexpected end of file, expected {expected}"),
            span,
        )
    }

    /// Creates an expected identifier error.
    pub fn expected_identifier(found: &Token) -> Self {
        let found_str = format!("{}", found.kind());
        Self::new(
            ParseErrorKind::ExpectedIdentifier {
                found: found_str.clone(),
            },
            format!("expected identifier, found {found_str}"),
            found.span(),
        )
    }

    /// Creates an unsupported feature error for deferred grammars.
    pub fn unsupported(feature: impl Into<String>, span: Span) -> Self {
        let feature = feature.into();
        Self::new(
            ParseErrorKind::UnsupportedFeature(feature.clone()),
            format!("{feature} is not implemented yet"),
            span,
        )
    }

    /// Converts this parser error into a structured [`sumer_diagnostics::Diagnostic`].
    pub fn to_diagnostic(&self) -> sumer_diagnostics::Diagnostic {
        sumer_diagnostics::Diagnostic::error(self.message.clone())
            .with_primary(self.span, self.message.clone())
    }
}

impl From<&ParseError> for sumer_diagnostics::Diagnostic {
    fn from(err: &ParseError) -> Self {
        err.to_diagnostic()
    }
}

impl From<ParseError> for sumer_diagnostics::Diagnostic {
    fn from(err: ParseError) -> Self {
        err.to_diagnostic()
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.message, self.span)
    }
}

impl Error for ParseError {}

/// Standard result type for parsing operations.
pub type ParseResult<T> = Result<T, ParseError>;
