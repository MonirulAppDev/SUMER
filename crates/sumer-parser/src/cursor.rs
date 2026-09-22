//! Token stream cursor and navigation utilities.

use sumer_ast::Identifier;
use sumer_lexer::{Token, TokenKind};

use crate::error::{ParseError, ParseResult};

/// Cursor over a slice of lexed tokens providing safe lookahead and consumption.
#[derive(Clone, Debug)]
pub struct TokenCursor<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl<'a> TokenCursor<'a> {
    /// Creates a new `TokenCursor` over the provided slice of tokens.
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// Returns the current token under the cursor without consuming it.
    ///
    /// If past the end of the token slice, returns the last token (guaranteed EOF).
    pub fn current(&self) -> &'a Token {
        if self.position < self.tokens.len() {
            &self.tokens[self.position]
        } else if let Some(last) = self.tokens.last() {
            last
        } else {
            panic!("empty token stream passed to parser");
        }
    }

    /// Synonym for [`current`](Self::current).
    pub fn peek(&self) -> &'a Token {
        self.current()
    }

    /// Looks ahead `n` tokens from the current position without advancing.
    ///
    /// `peek_n(0)` is equivalent to `peek()`.
    pub fn peek_n(&self, n: usize) -> &'a Token {
        let target = self.position + n;
        if target < self.tokens.len() {
            &self.tokens[target]
        } else if let Some(last) = self.tokens.last() {
            last
        } else {
            panic!("empty token stream passed to parser");
        }
    }

    /// Returns the previously consumed token, or current if at start.
    pub fn previous(&self) -> &'a Token {
        if self.position > 0 && self.position - 1 < self.tokens.len() {
            &self.tokens[self.position - 1]
        } else {
            self.current()
        }
    }

    /// Advances the cursor by one token and returns the consumed token.
    pub fn advance(&mut self) -> &'a Token {
        if !self.is_at_end() {
            let tok = &self.tokens[self.position];
            self.position += 1;
            tok
        } else {
            self.current()
        }
    }

    /// Returns `true` if the cursor is at the end of the token stream (`TokenKind::Eof`).
    pub fn is_at_end(&self) -> bool {
        self.current().kind() == &TokenKind::Eof || self.position >= self.tokens.len()
    }

    /// Checks if the current token matches the specified `TokenKind`.
    pub fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() && kind != &TokenKind::Eof {
            false
        } else {
            self.current().kind() == kind
        }
    }

    /// Consumes the current token if it matches `kind`, returning `true`.
    /// Otherwise returns `false` without advancing.
    pub fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consumes the current token if it matches `kind`, returning a reference to it.
    /// Returns a [`ParseError`] if the token does not match.
    pub fn expect(&mut self, kind: &TokenKind) -> ParseResult<&'a Token> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(ParseError::unexpected_token(
                format!("{kind}"),
                self.current(),
            ))
        }
    }

    /// Helper to consume an optional semicolon if present (SUMER optional semicolons).
    pub fn consume_semicolon_if_present(&mut self) -> bool {
        self.match_token(&TokenKind::Semicolon)
    }

    /// Parses an identifier from the current token, advancing if successful.
    pub fn parse_identifier(&mut self) -> ParseResult<Identifier> {
        let tok = self.current();
        if let TokenKind::Identifier(name) = tok.kind() {
            let ident = Identifier::new(name.clone(), tok.span());
            self.advance();
            Ok(ident)
        } else {
            Err(ParseError::expected_identifier(tok))
        }
    }

    /// Returns the current position index in the token stream.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Sets the position index in the token stream (used for backtracking).
    pub fn seek(&mut self, pos: usize) {
        self.position = pos.min(self.tokens.len());
    }
}
