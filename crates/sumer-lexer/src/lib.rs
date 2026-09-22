//! Lexer and token model for the SUMER programming language.
//!
//! This crate provides lexical analysis of SUMER source code (`.sm`),
//! producing a sequence of [`Token`]s with byte-level [`Span`](sumer_span::Span)s.
//!
//! # Main Types
//!
//! - [`Lexer`]: The lexical analyzer cursor.
//! - [`Token`]: A token paired with its source span.
//! - [`TokenKind`]: The category of token (ident, keyword, literal, operator, delimiter).
//! - [`LexerError`]: An error encountered during lexical analysis.
//! - [`lookup_keyword`]: Fast lookup for SUMER reserved keywords.

pub mod keyword;
pub mod lexer;
pub mod token;

pub use keyword::lookup_keyword;
pub use lexer::{Lexer, LexerError, LexerErrorKind};
pub use token::{Token, TokenKind};
