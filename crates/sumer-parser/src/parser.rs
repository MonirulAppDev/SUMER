//! Main parser entry point and driver.

use sumer_ast::Program;
use sumer_lexer::{Token, TokenKind};

use crate::attribute::parse_attributes;
use crate::cursor::TokenCursor;
use crate::declaration::parse_declaration;
use crate::error::ParseResult;

/// Main recursive descent parser for the SUMER language.
pub struct Parser<'a> {
    cursor: TokenCursor<'a>,
}

impl<'a> Parser<'a> {
    /// Creates a new `Parser` from a slice of tokens.
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            cursor: TokenCursor::new(tokens),
        }
    }

    /// Returns a reference to the underlying token cursor.
    pub fn cursor(&self) -> &TokenCursor<'a> {
        &self.cursor
    }

    /// Returns a mutable reference to the underlying token cursor.
    pub fn cursor_mut(&mut self) -> &mut TokenCursor<'a> {
        &mut self.cursor
    }

    /// Returns the current token under the cursor without consuming it.
    pub fn current(&self) -> &'a Token {
        self.cursor.current()
    }

    /// Synonym for [`current`](Self::current).
    pub fn peek(&self) -> &'a Token {
        self.cursor.peek()
    }

    /// Looks ahead `n` tokens from the current position without advancing.
    pub fn peek_n(&self, n: usize) -> &'a Token {
        self.cursor.peek_n(n)
    }

    /// Advances the cursor by one token and returns the consumed token.
    pub fn advance(&mut self) -> &'a Token {
        self.cursor.advance()
    }

    /// Returns `true` if the cursor is at the end of the token stream.
    pub fn is_at_end(&self) -> bool {
        self.cursor.is_at_end()
    }

    /// Checks if the current token matches the given `TokenKind`.
    pub fn check(&self, kind: &TokenKind) -> bool {
        self.cursor.check(kind)
    }

    /// Consumes the current token if it matches `kind`, returning `true`.
    pub fn match_token(&mut self, kind: &TokenKind) -> bool {
        self.cursor.match_token(kind)
    }

    /// Consumes the current token if it matches `kind`, or returns a [`ParseError`](crate::ParseError).
    pub fn expect(&mut self, kind: &TokenKind) -> ParseResult<&'a Token> {
        self.cursor.expect(kind)
    }

    /// Parses a complete SUMER [`Program`].
    pub fn parse_program(&mut self) -> ParseResult<Program> {
        let start_span = self.cursor.current().span();
        let mut file_attributes = Vec::new();
        let mut declarations = Vec::new();

        while !self.cursor.is_at_end() {
            // Tolerate optional top-level semicolons
            if self.cursor.match_token(&TokenKind::Semicolon) {
                continue;
            }

            let attrs = parse_attributes(&mut self.cursor)?;

            if self.cursor.is_at_end() {
                file_attributes.extend(attrs);
                break;
            }

            let decl = parse_declaration(&mut self.cursor, attrs)?;
            declarations.push(decl);
        }

        let end_span = self.cursor.current().span();
        let span = start_span.join(end_span).unwrap_or(start_span);

        Ok(Program::new(file_attributes, declarations, span))
    }
}

/// Convenience function to parse a token stream into an AST [`Program`].
pub fn parse(tokens: &[Token]) -> ParseResult<Program> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}
