//! Lexer implementation for the SUMER programming language.
//!
//! Transforms UTF-8 source text into a stream of [`Token`]s with accurate [`Span`] byte offsets.

use std::error::Error;
use std::fmt;

use sumer_span::{SourceId, Span};

use crate::keyword::lookup_keyword;
use crate::token::{Token, TokenKind};

/// Categories of lexer failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LexerErrorKind {
    /// An unexpected character was encountered.
    UnexpectedChar(char),
    /// A single-line string literal was unclosed before newline or EOF.
    UnterminatedString,
    /// A multiline string literal was unclosed before EOF.
    UnterminatedMultilineString,
    /// A character literal was unclosed before EOF.
    UnterminatedChar,
    /// A block comment was unclosed before EOF.
    UnterminatedBlockComment,
    /// An invalid numeric literal or prefix was encountered.
    InvalidNumber(String),
    /// An invalid escape sequence was found.
    InvalidEscape(char),
    /// An invalid character literal (e.g. empty or multiple codepoints).
    InvalidCharLiteral(String),
}

impl fmt::Display for LexerErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedChar(c) => write!(f, "unexpected character '{c}'"),
            Self::UnterminatedString => write!(f, "unterminated string literal"),
            Self::UnterminatedMultilineString => write!(f, "unterminated multiline string literal"),
            Self::UnterminatedChar => write!(f, "unterminated character literal"),
            Self::UnterminatedBlockComment => write!(f, "unterminated block comment"),
            Self::InvalidNumber(msg) => write!(f, "invalid numeric literal: {msg}"),
            Self::InvalidEscape(c) => write!(f, "invalid escape sequence '\\{c}'"),
            Self::InvalidCharLiteral(msg) => write!(f, "invalid character literal: {msg}"),
        }
    }
}

/// A lexical analysis error with source span location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexerError {
    /// The specific category of error.
    pub kind: LexerErrorKind,
    /// The exact byte span in the source file where the error occurred.
    pub span: Span,
    /// A human-readable diagnostic message.
    pub message: String,
}

impl LexerError {
    /// Creates a new `LexerError`.
    pub fn new(kind: LexerErrorKind, span: Span, message: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            message: message.into(),
        }
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.message, self.span)
    }
}

impl Error for LexerError {}

/// Lexical analyzer that breaks SUMER source code into tokens.
pub struct Lexer<'a> {
    source_id: SourceId,
    source: &'a str,
    cursor: usize,
    emitted_eof: bool,
}

impl<'a> Lexer<'a> {
    /// Creates a new `Lexer` for the given source file identifier and source code.
    pub fn new(source_id: SourceId, source: &'a str) -> Self {
        Self {
            source_id,
            source,
            cursor: 0,
            emitted_eof: false,
        }
    }

    /// Returns the source identifier being analyzed.
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    /// Returns the current byte cursor position in the source text.
    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    /// Peeks at the next Unicode character without consuming it.
    fn peek(&self) -> Option<char> {
        self.source[self.cursor..].chars().next()
    }

    /// Peeks at the `n`-th Unicode character ahead (0-indexed).
    fn peek_nth(&self, n: usize) -> Option<char> {
        self.source[self.cursor..].chars().nth(n)
    }

    /// Checks if the remaining source begins with the given prefix.
    fn starts_with(&self, prefix: &str) -> bool {
        self.source[self.cursor..].starts_with(prefix)
    }

    /// Consumes the next Unicode character and advances the byte cursor.
    fn bump(&mut self) -> Option<char> {
        let ch = self.source[self.cursor..].chars().next()?;
        self.cursor += ch.len_utf8();
        Some(ch)
    }

    /// Skips whitespace and single/multi-line/doc comments.
    fn skip_whitespace_and_comments(&mut self) -> Result<(), LexerError> {
        loop {
            match self.peek() {
                Some(' ' | '\t' | '\r' | '\n') => {
                    self.bump();
                }
                Some('/') => {
                    if self.starts_with("///") {
                        // Documentation comment: skip until newline or EOF while preserving position
                        self.skip_line_comment();
                    } else if self.starts_with("//") {
                        // Single-line comment
                        self.skip_line_comment();
                    } else if self.starts_with("/*") {
                        // Multi-line block comment
                        self.skip_block_comment()?;
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }

    /// Skips single-line comment characters until `\n` or EOF.
    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.bump();
        }
    }

    /// Skips nested multi-line block comments (`/* ... */`).
    fn skip_block_comment(&mut self) -> Result<(), LexerError> {
        let start = self.cursor as u32;
        self.bump(); // '/'
        self.bump(); // '*'
        let mut depth = 1;

        while depth > 0 {
            if self.starts_with("/*") {
                self.bump();
                self.bump();
                depth += 1;
            } else if self.starts_with("*/") {
                self.bump();
                self.bump();
                depth -= 1;
            } else if self.bump().is_none() {
                let end = self.cursor as u32;
                return Err(LexerError::new(
                    LexerErrorKind::UnterminatedBlockComment,
                    Span::new(self.source_id, start, end),
                    "unterminated block comment",
                ));
            }
        }
        Ok(())
    }

    /// Reads an identifier or reserved keyword.
    fn read_identifier_or_keyword(&mut self) -> Token {
        let start = self.cursor as u32;
        self.bump(); // consume leading letter or '_'

        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.bump();
            } else {
                break;
            }
        }

        let end = self.cursor as u32;
        let text = &self.source[start as usize..end as usize];
        let span = Span::new(self.source_id, start, end);

        let kind = lookup_keyword(text).unwrap_or_else(|| TokenKind::Identifier(text.to_string()));
        Token::new(kind, span)
    }

    /// Reads an integer or floating-point literal.
    fn read_number(&mut self) -> Result<Token, LexerError> {
        let start = self.cursor as u32;
        let first = self.bump().unwrap();

        // Check for base prefixes: 0x (hex), 0b (bin), 0o (oct)
        if first == '0' {
            match self.peek() {
                Some('x' | 'X') => {
                    self.bump();
                    let mut digit_count = 0;
                    while let Some(c) = self.peek() {
                        if c.is_ascii_hexdigit() {
                            self.bump();
                            digit_count += 1;
                        } else if c == '_' {
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    if digit_count == 0 {
                        let end = self.cursor as u32;
                        return Err(LexerError::new(
                            LexerErrorKind::InvalidNumber(
                                "expected hexadecimal digits after '0x'".to_string(),
                            ),
                            Span::new(self.source_id, start, end),
                            "expected hexadecimal digits after '0x'",
                        ));
                    }
                    if let Some(c) = self.peek().filter(|c| c.is_ascii_alphabetic()) {
                        let end = (self.cursor + c.len_utf8()) as u32;
                        return Err(LexerError::new(
                            LexerErrorKind::InvalidNumber(format!(
                                "invalid character '{c}' in hexadecimal literal"
                            )),
                            Span::new(self.source_id, start, end),
                            format!("invalid character '{c}' in hexadecimal literal"),
                        ));
                    }
                    let end = self.cursor as u32;
                    let val = self.source[start as usize..end as usize].to_string();
                    return Ok(Token::new(
                        TokenKind::Integer(val),
                        Span::new(self.source_id, start, end),
                    ));
                }
                Some('b' | 'B') => {
                    self.bump();
                    let mut digit_count = 0;
                    while let Some(c) = self.peek() {
                        if c == '0' || c == '1' {
                            self.bump();
                            digit_count += 1;
                        } else if c == '_' {
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    if digit_count == 0 {
                        let end = self.cursor as u32;
                        return Err(LexerError::new(
                            LexerErrorKind::InvalidNumber(
                                "expected binary digits after '0b'".to_string(),
                            ),
                            Span::new(self.source_id, start, end),
                            "expected binary digits after '0b'",
                        ));
                    }
                    if let Some(c) = self.peek().filter(|c| c.is_ascii_alphanumeric()) {
                        let end = (self.cursor + c.len_utf8()) as u32;
                        return Err(LexerError::new(
                            LexerErrorKind::InvalidNumber(format!(
                                "invalid character '{c}' in binary literal"
                            )),
                            Span::new(self.source_id, start, end),
                            format!("invalid character '{c}' in binary literal"),
                        ));
                    }
                    let end = self.cursor as u32;
                    let val = self.source[start as usize..end as usize].to_string();
                    return Ok(Token::new(
                        TokenKind::Integer(val),
                        Span::new(self.source_id, start, end),
                    ));
                }
                Some('o' | 'O') => {
                    self.bump();
                    let mut digit_count = 0;
                    while let Some(c) = self.peek() {
                        if ('0'..='7').contains(&c) {
                            self.bump();
                            digit_count += 1;
                        } else if c == '_' {
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    if digit_count == 0 {
                        let end = self.cursor as u32;
                        return Err(LexerError::new(
                            LexerErrorKind::InvalidNumber(
                                "expected octal digits after '0o'".to_string(),
                            ),
                            Span::new(self.source_id, start, end),
                            "expected octal digits after '0o'",
                        ));
                    }
                    if let Some(c) = self.peek().filter(|c| c.is_ascii_alphanumeric()) {
                        let end = (self.cursor + c.len_utf8()) as u32;
                        return Err(LexerError::new(
                            LexerErrorKind::InvalidNumber(format!(
                                "invalid character '{c}' in octal literal"
                            )),
                            Span::new(self.source_id, start, end),
                            format!("invalid character '{c}' in octal literal"),
                        ));
                    }
                    let end = self.cursor as u32;
                    let val = self.source[start as usize..end as usize].to_string();
                    return Ok(Token::new(
                        TokenKind::Integer(val),
                        Span::new(self.source_id, start, end),
                    ));
                }
                _ => {}
            }
        }

        // Decimal digits
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '_' {
                self.bump();
            } else {
                break;
            }
        }

        // Float check: a '.' only forms a float if followed by a digit (NOT `..` or `..=`)
        let mut is_float = false;
        if self.peek() == Some('.') && matches!(self.peek_nth(1), Some(c) if c.is_ascii_digit()) {
            is_float = true;
            self.bump(); // consume '.'
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == '_' {
                    self.bump();
                } else {
                    break;
                }
            }
        }

        // Exponent check (e.g. 1e10, 1.5e-3)
        if self.peek() == Some('e') || self.peek() == Some('E') {
            let next1 = self.peek_nth(1);
            let next2 = self.peek_nth(2);
            let valid_exp = match (next1, next2) {
                (Some('+' | '-'), Some(d)) if d.is_ascii_digit() => true,
                (Some(d), _) if d.is_ascii_digit() => true,
                _ => false,
            };
            if valid_exp {
                is_float = true;
                self.bump(); // consume 'e' or 'E'
                if self.peek() == Some('+') || self.peek() == Some('-') {
                    self.bump();
                }
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() || c == '_' {
                        self.bump();
                    } else {
                        break;
                    }
                }
            }
        }

        // Reject immediate identifier characters following a number (e.g. `1name`)
        if self
            .peek()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        {
            let err_start = start;
            while let Some(c) = self.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    self.bump();
                } else {
                    break;
                }
            }
            let end = self.cursor as u32;
            let bad = &self.source[err_start as usize..end as usize];
            return Err(LexerError::new(
                LexerErrorKind::InvalidNumber(format!("invalid numeric literal '{bad}'")),
                Span::new(self.source_id, err_start, end),
                format!("identifier cannot immediately follow numeric literal '{bad}'"),
            ));
        }

        let end = self.cursor as u32;
        let val = self.source[start as usize..end as usize].to_string();
        let span = Span::new(self.source_id, start, end);

        if is_float {
            Ok(Token::new(TokenKind::Float(val), span))
        } else {
            Ok(Token::new(TokenKind::Integer(val), span))
        }
    }

    /// Reads a single-line or multiline string literal.
    fn read_string(&mut self) -> Result<Token, LexerError> {
        let start = self.cursor as u32;

        // Multiline string: `"""`
        if self.starts_with("\"\"\"") {
            self.bump();
            self.bump();
            self.bump();
            let mut content = String::new();

            loop {
                if self.starts_with("\"\"\"") {
                    self.bump();
                    self.bump();
                    self.bump();
                    let end = self.cursor as u32;
                    return Ok(Token::new(
                        TokenKind::String(content),
                        Span::new(self.source_id, start, end),
                    ));
                }
                match self.bump() {
                    Some(c) => content.push(c),
                    None => {
                        let end = self.cursor as u32;
                        return Err(LexerError::new(
                            LexerErrorKind::UnterminatedMultilineString,
                            Span::new(self.source_id, start, end),
                            "unterminated multiline string literal",
                        ));
                    }
                }
            }
        }

        // Single-line string: `"`
        self.bump(); // consume opening `"`
        let mut content = String::new();

        loop {
            match self.peek() {
                Some('"') => {
                    self.bump(); // consume closing `"`
                    let end = self.cursor as u32;
                    return Ok(Token::new(
                        TokenKind::String(content),
                        Span::new(self.source_id, start, end),
                    ));
                }
                Some('\\') => {
                    self.bump(); // consume `\`
                    match self.bump() {
                        Some('n') => content.push('\n'),
                        Some('r') => content.push('\r'),
                        Some('t') => content.push('\t'),
                        Some('\\') => content.push('\\'),
                        Some('"') => content.push('"'),
                        Some('\'') => content.push('\''),
                        Some('0') => content.push('\0'),
                        Some(bad) => {
                            let end = self.cursor as u32;
                            return Err(LexerError::new(
                                LexerErrorKind::InvalidEscape(bad),
                                Span::new(self.source_id, start, end),
                                format!("invalid escape sequence '\\{bad}'"),
                            ));
                        }
                        None => {
                            let end = self.cursor as u32;
                            return Err(LexerError::new(
                                LexerErrorKind::UnterminatedString,
                                Span::new(self.source_id, start, end),
                                "unterminated string literal",
                            ));
                        }
                    }
                }
                Some('\n') | None => {
                    let end = self.cursor as u32;
                    return Err(LexerError::new(
                        LexerErrorKind::UnterminatedString,
                        Span::new(self.source_id, start, end),
                        "unterminated string literal",
                    ));
                }
                Some(c) => {
                    self.bump();
                    content.push(c);
                }
            }
        }
    }

    /// Reads a character literal enclosed in single quotes `'c'`.
    fn read_char(&mut self) -> Result<Token, LexerError> {
        let start = self.cursor as u32;
        self.bump(); // consume opening quote `'`

        if self.peek() == Some('\'') {
            self.bump();
            let end = self.cursor as u32;
            return Err(LexerError::new(
                LexerErrorKind::InvalidCharLiteral("empty character literal".to_string()),
                Span::new(self.source_id, start, end),
                "empty character literal",
            ));
        }

        let ch = match self.peek() {
            Some('\\') => {
                self.bump(); // consume `\`
                match self.bump() {
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    Some('\\') => '\\',
                    Some('\'') => '\'',
                    Some('"') => '"',
                    Some('0') => '\0',
                    Some(bad) => {
                        let end = self.cursor as u32;
                        return Err(LexerError::new(
                            LexerErrorKind::InvalidEscape(bad),
                            Span::new(self.source_id, start, end),
                            format!("invalid escape sequence '\\{bad}'"),
                        ));
                    }
                    None => {
                        let end = self.cursor as u32;
                        return Err(LexerError::new(
                            LexerErrorKind::UnterminatedChar,
                            Span::new(self.source_id, start, end),
                            "unterminated character literal",
                        ));
                    }
                }
            }
            Some('\n') | None => {
                let end = self.cursor as u32;
                return Err(LexerError::new(
                    LexerErrorKind::UnterminatedChar,
                    Span::new(self.source_id, start, end),
                    "unterminated character literal",
                ));
            }
            Some(_) => self.bump().unwrap(),
        };

        if self.peek() == Some('\'') {
            self.bump(); // consume closing `'`
            let end = self.cursor as u32;
            Ok(Token::new(
                TokenKind::Char(ch),
                Span::new(self.source_id, start, end),
            ))
        } else if self.peek().is_none() || self.peek() == Some('\n') {
            let end = self.cursor as u32;
            Err(LexerError::new(
                LexerErrorKind::UnterminatedChar,
                Span::new(self.source_id, start, end),
                "unterminated character literal",
            ))
        } else {
            let mut closed = false;
            while let Some(c) = self.peek() {
                if c == '\'' {
                    self.bump();
                    closed = true;
                    break;
                }
                if c == '\n' {
                    break;
                }
                self.bump();
            }
            let end = self.cursor as u32;
            if closed {
                Err(LexerError::new(
                    LexerErrorKind::InvalidCharLiteral(
                        "character literal may only contain one codepoint".to_string(),
                    ),
                    Span::new(self.source_id, start, end),
                    "character literal may only contain one codepoint",
                ))
            } else {
                Err(LexerError::new(
                    LexerErrorKind::UnterminatedChar,
                    Span::new(self.source_id, start, end),
                    "unterminated character literal",
                ))
            }
        }
    }

    /// Fetches the next token from the source stream.
    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace_and_comments()?;

        let start = self.cursor as u32;

        let Some(ch) = self.peek() else {
            self.emitted_eof = true;
            let len = self.source.len() as u32;
            return Ok(Token::new(
                TokenKind::Eof,
                Span::new(self.source_id, len, len),
            ));
        };

        // Identifiers and Keywords
        if ch.is_ascii_alphabetic() || ch == '_' {
            return Ok(self.read_identifier_or_keyword());
        }

        // Numeric Literals
        if ch.is_ascii_digit() {
            return self.read_number();
        }

        // String Literals
        if ch == '"' {
            return self.read_string();
        }

        // Character Literals
        if ch == '\'' {
            return self.read_char();
        }

        // Operators & Delimiters
        self.bump(); // consume the first character

        let kind = match ch {
            // Arithmetic
            '+' => {
                if self.peek() == Some('=') {
                    self.bump();
                    TokenKind::PlusEqual
                } else {
                    TokenKind::Plus
                }
            }
            '-' => {
                if self.peek() == Some('=') {
                    self.bump();
                    TokenKind::MinusEqual
                } else if self.peek() == Some('>') {
                    self.bump();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                if self.peek() == Some('=') {
                    self.bump();
                    TokenKind::StarEqual
                } else {
                    TokenKind::Star
                }
            }
            '/' => {
                if self.peek() == Some('=') {
                    self.bump();
                    TokenKind::SlashEqual
                } else {
                    TokenKind::Slash
                }
            }
            '%' => {
                if self.peek() == Some('=') {
                    self.bump();
                    TokenKind::PercentEqual
                } else {
                    TokenKind::Percent
                }
            }

            // Assignment & Lambda & Comparison
            '=' => {
                if self.peek() == Some('=') {
                    self.bump();
                    TokenKind::EqualEqual
                } else if self.peek() == Some('>') {
                    self.bump();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Equal
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.bump();
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.bump();
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.bump();
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            }

            // Logical & Reference
            '&' => {
                if self.peek() == Some('&') {
                    self.bump();
                    TokenKind::AmpAmp
                } else if self.starts_with("mut") {
                    // Check if char after "mut" is not an identifier character
                    let next_char = self.peek_nth(3);
                    let is_ident_continue =
                        matches!(next_char, Some(c) if c.is_ascii_alphanumeric() || c == '_');
                    if !is_ident_continue {
                        self.bump(); // 'm'
                        self.bump(); // 'u'
                        self.bump(); // 't'
                        TokenKind::AmpMut
                    } else {
                        TokenKind::Ampersand
                    }
                } else {
                    TokenKind::Ampersand
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.bump();
                    TokenKind::PipePipe
                } else {
                    let end = self.cursor as u32;
                    return Err(LexerError::new(
                        LexerErrorKind::UnexpectedChar('|'),
                        Span::new(self.source_id, start, end),
                        "unexpected character '|'",
                    ));
                }
            }
            '?' => TokenKind::Question,

            // Range & Dot
            '.' => {
                if self.peek() == Some('.') {
                    self.bump();
                    if self.peek() == Some('=') {
                        self.bump();
                        TokenKind::DotDotEqual
                    } else {
                        TokenKind::DotDot
                    }
                } else {
                    TokenKind::Dot
                }
            }

            // Delimiters
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ':' => TokenKind::Colon,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            '@' => TokenKind::At,

            // Unknown characters
            unknown => {
                let end = self.cursor as u32;
                return Err(LexerError::new(
                    LexerErrorKind::UnexpectedChar(unknown),
                    Span::new(self.source_id, start, end),
                    format!("unexpected character '{unknown}'"),
                ));
            }
        };

        let end = self.cursor as u32;
        Ok(Token::new(kind, Span::new(self.source_id, start, end)))
    }

    /// Lexes the entire source code into a vector of [`Token`]s including [`TokenKind::Eof`].
    pub fn lex(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.emitted_eof {
            return None;
        }

        match self.next_token() {
            Ok(tok) => {
                if tok.kind == TokenKind::Eof {
                    self.emitted_eof = true;
                }
                Some(Ok(tok))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_all(source: &str) -> Result<Vec<Token>, LexerError> {
        Lexer::new(SourceId(0), source).lex()
    }

    fn kinds(source: &str) -> Vec<TokenKind> {
        lex_all(source)
            .expect("lexing succeeded")
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    // ----------------------------------------------------
    // BASIC
    // ----------------------------------------------------

    #[test]
    fn test_empty_source() {
        let tokens = lex_all("").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
        assert_eq!(tokens[0].span, Span::new(SourceId(0), 0, 0));
    }

    #[test]
    fn test_eof_span() {
        let tokens = lex_all("abc").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[1].kind, TokenKind::Eof);
        assert_eq!(tokens[1].span, Span::new(SourceId(0), 3, 3));
    }

    #[test]
    fn test_identifier() {
        let k = kinds("hello_world");
        assert_eq!(
            k,
            vec![
                TokenKind::Identifier("hello_world".to_string()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_multiple_identifiers() {
        let k = kinds("user name main calculate User MyStruct");
        assert_eq!(
            k,
            vec![
                TokenKind::Identifier("user".to_string()),
                TokenKind::Identifier("name".to_string()),
                TokenKind::Identifier("main".to_string()),
                TokenKind::Identifier("calculate".to_string()),
                TokenKind::Identifier("User".to_string()),
                TokenKind::Identifier("MyStruct".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_boolean() {
        let k = kinds("true false");
        assert_eq!(k, vec![TokenKind::True, TokenKind::False, TokenKind::Eof]);
    }

    // ----------------------------------------------------
    // KEYWORDS
    // ----------------------------------------------------

    #[test]
    fn test_all_keywords() {
        let src = "let var const fn return struct enum trait impl if else for in while loop match break continue import pub async await spawn unsafe true false Some None Ok Err";
        let k = kinds(src);
        assert_eq!(
            k,
            vec![
                TokenKind::Let,
                TokenKind::Var,
                TokenKind::Const,
                TokenKind::Fn,
                TokenKind::Return,
                TokenKind::Struct,
                TokenKind::Enum,
                TokenKind::Trait,
                TokenKind::Impl,
                TokenKind::If,
                TokenKind::Else,
                TokenKind::For,
                TokenKind::In,
                TokenKind::While,
                TokenKind::Loop,
                TokenKind::Match,
                TokenKind::Break,
                TokenKind::Continue,
                TokenKind::Import,
                TokenKind::Pub,
                TokenKind::Async,
                TokenKind::Await,
                TokenKind::Spawn,
                TokenKind::Unsafe,
                TokenKind::True,
                TokenKind::False,
                TokenKind::Some,
                TokenKind::None,
                TokenKind::Ok,
                TokenKind::Err,
                TokenKind::Eof,
            ]
        );
    }

    // ----------------------------------------------------
    // OPERATORS
    // ----------------------------------------------------

    #[test]
    fn test_all_operators() {
        let src = "+ - * / % = == != > < >= <= && || ! += -= *= /= %= & &mut ? .. ..= => ->";
        let k = kinds(src);
        assert_eq!(
            k,
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Percent,
                TokenKind::Equal,
                TokenKind::EqualEqual,
                TokenKind::BangEqual,
                TokenKind::Greater,
                TokenKind::Less,
                TokenKind::GreaterEqual,
                TokenKind::LessEqual,
                TokenKind::AmpAmp,
                TokenKind::PipePipe,
                TokenKind::Bang,
                TokenKind::PlusEqual,
                TokenKind::MinusEqual,
                TokenKind::StarEqual,
                TokenKind::SlashEqual,
                TokenKind::PercentEqual,
                TokenKind::Ampersand,
                TokenKind::AmpMut,
                TokenKind::Question,
                TokenKind::DotDot,
                TokenKind::DotDotEqual,
                TokenKind::FatArrow,
                TokenKind::Arrow,
                TokenKind::Eof,
            ]
        );
    }

    // ----------------------------------------------------
    // DELIMITERS
    // ----------------------------------------------------

    #[test]
    fn test_all_delimiters() {
        let src = "( ) { } [ ] : , . ;";
        let k = kinds(src);
        assert_eq!(
            k,
            vec![
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::LBrace,
                TokenKind::RBrace,
                TokenKind::LBracket,
                TokenKind::RBracket,
                TokenKind::Colon,
                TokenKind::Comma,
                TokenKind::Dot,
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    // ----------------------------------------------------
    // COMMENTS
    // ----------------------------------------------------

    #[test]
    fn test_comments() {
        let src = r#"
            // single line comment
            let x = 10 // end of line comment
            /* block comment */
            let y = /* inline block */ 20
            /* multi
               line
               comment */
            /// documentation comment
            let z = 30
        "#;
        let k = kinds(src);
        assert_eq!(
            k,
            vec![
                TokenKind::Let,
                TokenKind::Identifier("x".to_string()),
                TokenKind::Equal,
                TokenKind::Integer("10".to_string()),
                TokenKind::Let,
                TokenKind::Identifier("y".to_string()),
                TokenKind::Equal,
                TokenKind::Integer("20".to_string()),
                TokenKind::Let,
                TokenKind::Identifier("z".to_string()),
                TokenKind::Equal,
                TokenKind::Integer("30".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    // ----------------------------------------------------
    // NUMBERS & RANGE EDGE CASES
    // ----------------------------------------------------

    #[test]
    fn test_numbers() {
        let src = "123 1_000 0xff 0b1010 0o755 3.14 0.5 1e10 1.5e-3";
        let k = kinds(src);
        assert_eq!(
            k,
            vec![
                TokenKind::Integer("123".to_string()),
                TokenKind::Integer("1_000".to_string()),
                TokenKind::Integer("0xff".to_string()),
                TokenKind::Integer("0b1010".to_string()),
                TokenKind::Integer("0o755".to_string()),
                TokenKind::Float("3.14".to_string()),
                TokenKind::Float("0.5".to_string()),
                TokenKind::Float("1e10".to_string()),
                TokenKind::Float("1.5e-3".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_range_edge_cases_not_floats() {
        // 1..10 must lex as Integer(1), Range(..), Integer(10)
        assert_eq!(
            kinds("1..10"),
            vec![
                TokenKind::Integer("1".to_string()),
                TokenKind::DotDot,
                TokenKind::Integer("10".to_string()),
                TokenKind::Eof,
            ]
        );

        // 1..=10 must lex as Integer(1), RangeInclusive(..=), Integer(10)
        assert_eq!(
            kinds("1..=10"),
            vec![
                TokenKind::Integer("1".to_string()),
                TokenKind::DotDotEqual,
                TokenKind::Integer("10".to_string()),
                TokenKind::Eof,
            ]
        );

        assert_eq!(
            kinds("10..20"),
            vec![
                TokenKind::Integer("10".to_string()),
                TokenKind::DotDot,
                TokenKind::Integer("20".to_string()),
                TokenKind::Eof,
            ]
        );

        assert_eq!(
            kinds("10..=20"),
            vec![
                TokenKind::Integer("10".to_string()),
                TokenKind::DotDotEqual,
                TokenKind::Integer("20".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    // ----------------------------------------------------
    // STRINGS & MULTILINE STRINGS
    // ----------------------------------------------------

    #[test]
    fn test_strings() {
        let src = r#""hello" "hello world" "Hello {name}""#;
        let k = kinds(src);
        assert_eq!(
            k,
            vec![
                TokenKind::String("hello".to_string()),
                TokenKind::String("hello world".to_string()),
                TokenKind::String("Hello {name}".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_multiline_string() {
        let src = "\"\"\"\nhello\nSUMER\n\"\"\"";
        let k = kinds(src);
        assert_eq!(
            k,
            vec![
                TokenKind::String("\nhello\nSUMER\n".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_utf8_strings() {
        let src = "\"বাংলা\"";
        let tokens = lex_all(src).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::String("বাংলা".to_string()));
        assert_eq!(tokens[0].span, Span::new(SourceId(0), 0, src.len() as u32));
    }

    // ----------------------------------------------------
    // CHARACTERS
    // ----------------------------------------------------

    #[test]
    fn test_characters() {
        let src = "'a' '\\n' '\\t' '\\\\'";
        let k = kinds(src);
        assert_eq!(
            k,
            vec![
                TokenKind::Char('a'),
                TokenKind::Char('\n'),
                TokenKind::Char('\t'),
                TokenKind::Char('\\'),
                TokenKind::Eof,
            ]
        );
    }

    // ----------------------------------------------------
    // IDENTIFIER EDGE CASES
    // ----------------------------------------------------

    #[test]
    fn test_identifier_edge_cases() {
        let src = "name _name name1 MyStruct calculateValue";
        let k = kinds(src);
        assert_eq!(
            k,
            vec![
                TokenKind::Identifier("name".to_string()),
                TokenKind::Identifier("_name".to_string()),
                TokenKind::Identifier("name1".to_string()),
                TokenKind::Identifier("MyStruct".to_string()),
                TokenKind::Identifier("calculateValue".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_reject_1name() {
        let err = lex_all("1name").unwrap_err();
        assert!(matches!(err.kind, LexerErrorKind::InvalidNumber(_)));
    }

    // ----------------------------------------------------
    // ERRORS
    // ----------------------------------------------------

    #[test]
    fn test_unexpected_character() {
        let err = lex_all("$").unwrap_err();
        assert_eq!(err.kind, LexerErrorKind::UnexpectedChar('$'));
        assert_eq!(err.span, Span::new(SourceId(0), 0, 1));

        let err2 = lex_all("#").unwrap_err();
        assert_eq!(err2.kind, LexerErrorKind::UnexpectedChar('#'));
    }

    #[test]
    fn test_unterminated_string() {
        let err = lex_all("\"unterminated").unwrap_err();
        assert_eq!(err.kind, LexerErrorKind::UnterminatedString);
    }

    #[test]
    fn test_unterminated_multiline_string() {
        let err = lex_all("\"\"\"hello").unwrap_err();
        assert_eq!(err.kind, LexerErrorKind::UnterminatedMultilineString);
    }

    #[test]
    fn test_unterminated_character() {
        let err = lex_all("'a").unwrap_err();
        assert_eq!(err.kind, LexerErrorKind::UnterminatedChar);
    }

    #[test]
    fn test_unterminated_block_comment() {
        let err = lex_all("/* unclosed").unwrap_err();
        assert_eq!(err.kind, LexerErrorKind::UnterminatedBlockComment);
    }

    #[test]
    fn test_invalid_escape() {
        let err = lex_all("\"bad \\q escape\"").unwrap_err();
        assert_eq!(err.kind, LexerErrorKind::InvalidEscape('q'));
    }

    #[test]
    fn test_invalid_numeric_literal() {
        let err = lex_all("0x").unwrap_err();
        assert!(matches!(err.kind, LexerErrorKind::InvalidNumber(_)));
    }

    // ----------------------------------------------------
    // SOURCE SPANS & CRLF
    // ----------------------------------------------------

    #[test]
    fn test_source_spans() {
        let src = "fn main()";
        let tokens = lex_all(src).unwrap();
        assert_eq!(tokens[0].span, Span::new(SourceId(0), 0, 2)); // "fn"
        assert_eq!(tokens[1].span, Span::new(SourceId(0), 3, 7)); // "main"
        assert_eq!(tokens[2].span, Span::new(SourceId(0), 7, 8)); // "("
        assert_eq!(tokens[3].span, Span::new(SourceId(0), 8, 9)); // ")"
        assert_eq!(tokens[4].span, Span::new(SourceId(0), 9, 9)); // EOF
    }

    #[test]
    fn test_crlf_handling() {
        let src = "fn main() {\r\n    print(\"Hello\")\r\n}";
        let tokens = lex_all(src).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("main".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::LParen);
        assert_eq!(tokens[3].kind, TokenKind::RParen);
        assert_eq!(tokens[4].kind, TokenKind::LBrace);
        assert_eq!(tokens[5].kind, TokenKind::Identifier("print".to_string()));
        assert_eq!(tokens[6].kind, TokenKind::LParen);
        assert_eq!(tokens[7].kind, TokenKind::String("Hello".to_string()));
        assert_eq!(tokens[8].kind, TokenKind::RParen);
        assert_eq!(tokens[9].kind, TokenKind::RBrace);
        assert_eq!(tokens[10].kind, TokenKind::Eof);
    }

    // ----------------------------------------------------
    // FULL SUMER REALISTIC PROGRAM
    // ----------------------------------------------------

    #[test]
    fn test_full_sumer_realistic_program() {
        let src = r#"
fn main() {
    let name = "Monir"
    let age = 30

    if age >= 18 {
        print("Adult")
    } else {
        print("Minor")
    }
}
"#;
        let tokens = lex_all(src).unwrap();
        let expected_kinds = vec![
            TokenKind::Fn,
            TokenKind::Identifier("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::Let,
            TokenKind::Identifier("name".to_string()),
            TokenKind::Equal,
            TokenKind::String("Monir".to_string()),
            TokenKind::Let,
            TokenKind::Identifier("age".to_string()),
            TokenKind::Equal,
            TokenKind::Integer("30".to_string()),
            TokenKind::If,
            TokenKind::Identifier("age".to_string()),
            TokenKind::GreaterEqual,
            TokenKind::Integer("18".to_string()),
            TokenKind::LBrace,
            TokenKind::Identifier("print".to_string()),
            TokenKind::LParen,
            TokenKind::String("Adult".to_string()),
            TokenKind::RParen,
            TokenKind::RBrace,
            TokenKind::Else,
            TokenKind::LBrace,
            TokenKind::Identifier("print".to_string()),
            TokenKind::LParen,
            TokenKind::String("Minor".to_string()),
            TokenKind::RParen,
            TokenKind::RBrace,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let actual_kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
        assert_eq!(actual_kinds, expected_kinds);
    }

    #[test]
    fn test_sample_tokenization_hello_sm() {
        let src = "fn main() {\n    print(\"Hello, SUMER!\")\n}";
        let tokens = lex_all(src).unwrap();
        let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Fn,
                TokenKind::Identifier("main".to_string()),
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::LBrace,
                TokenKind::Identifier("print".to_string()),
                TokenKind::LParen,
                TokenKind::String("Hello, SUMER!".to_string()),
                TokenKind::RParen,
                TokenKind::RBrace,
                TokenKind::Eof,
            ]
        );
    }
}
