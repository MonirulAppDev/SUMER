//! Token definitions and kinds for the SUMER programming language.

use std::fmt;
use sumer_span::Span;

/// A single lexical token with its kind and source span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    /// The specific category of this token.
    pub kind: TokenKind,
    /// The byte-offset span in the source code where this token appears.
    pub span: Span,
}

impl Token {
    /// Creates a new `Token`.
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Returns the kind of the token.
    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    /// Returns the source span of the token.
    pub const fn span(&self) -> Span {
        self.span
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.kind, self.span)
    }
}

/// Enumeration of all token categories in the SUMER v0.1 grammar.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // ----------------------------------------------------
    // Identifiers & Literals
    // ----------------------------------------------------
    /// Identifier (variable names, function names, types).
    Identifier(String),
    /// Integer literal in source representation (e.g. "123", "0xFF", "0b1010", "0o755").
    Integer(String),
    /// Floating-point literal (e.g. "3.14", "1e10", "1.5e-3").
    Float(String),
    /// String literal content (e.g. "Hello, SUMER!").
    String(String),
    /// Character literal (e.g. 'a', '\n').
    Char(char),

    // ----------------------------------------------------
    // Keywords
    // ----------------------------------------------------
    Let,
    Var,
    Const,
    Fn,
    Return,
    Struct,
    Enum,
    Trait,
    Impl,
    If,
    Else,
    For,
    In,
    While,
    Loop,
    Match,
    Break,
    Continue,
    Import,
    Pub,
    Async,
    Await,
    Spawn,
    Unsafe,
    True,
    False,
    Some,
    None,
    Ok,
    Err,

    // ----------------------------------------------------
    // Operators
    // ----------------------------------------------------
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,

    /// `==`
    EqualEqual,
    /// `!=`
    BangEqual,
    /// `>`
    Greater,
    /// `<`
    Less,
    /// `>=`
    GreaterEqual,
    /// `<=`
    LessEqual,

    /// `&&`
    AmpAmp,
    /// `||`
    PipePipe,
    /// `!`
    Bang,

    /// `=`
    Equal,
    /// `+=`
    PlusEqual,
    /// `-=`
    MinusEqual,
    /// `*=`
    StarEqual,
    /// `/=`
    SlashEqual,
    /// `%=`
    PercentEqual,

    /// `&`
    Ampersand,
    /// `&mut`
    AmpMut,
    /// `?`
    Question,
    /// `..`
    DotDot,
    /// `..=`
    DotDotEqual,
    /// `=>`
    FatArrow,
    /// `->`
    Arrow,

    // ----------------------------------------------------
    // Delimiters
    // ----------------------------------------------------
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `;`
    Semicolon,
    /// `@`
    At,

    // ----------------------------------------------------
    // Special
    // ----------------------------------------------------
    /// End of File.
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identifier(name) => write!(f, "Identifier(\"{name}\")"),
            Self::Integer(val) => write!(f, "Integer({val})"),
            Self::Float(val) => write!(f, "Float({val})"),
            Self::String(val) => write!(f, "String(\"{val}\")"),
            Self::Char(c) => write!(f, "Char('{c}')"),

            Self::Let => write!(f, "let"),
            Self::Var => write!(f, "var"),
            Self::Const => write!(f, "const"),
            Self::Fn => write!(f, "fn"),
            Self::Return => write!(f, "return"),
            Self::Struct => write!(f, "struct"),
            Self::Enum => write!(f, "enum"),
            Self::Trait => write!(f, "trait"),
            Self::Impl => write!(f, "impl"),
            Self::If => write!(f, "if"),
            Self::Else => write!(f, "else"),
            Self::For => write!(f, "for"),
            Self::In => write!(f, "in"),
            Self::While => write!(f, "while"),
            Self::Loop => write!(f, "loop"),
            Self::Match => write!(f, "match"),
            Self::Break => write!(f, "break"),
            Self::Continue => write!(f, "continue"),
            Self::Import => write!(f, "import"),
            Self::Pub => write!(f, "pub"),
            Self::Async => write!(f, "async"),
            Self::Await => write!(f, "await"),
            Self::Spawn => write!(f, "spawn"),
            Self::Unsafe => write!(f, "unsafe"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Some => write!(f, "Some"),
            Self::None => write!(f, "None"),
            Self::Ok => write!(f, "Ok"),
            Self::Err => write!(f, "Err"),

            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::Percent => write!(f, "%"),
            Self::EqualEqual => write!(f, "=="),
            Self::BangEqual => write!(f, "!="),
            Self::Greater => write!(f, ">"),
            Self::Less => write!(f, "<"),
            Self::GreaterEqual => write!(f, ">="),
            Self::LessEqual => write!(f, "<="),
            Self::AmpAmp => write!(f, "&&"),
            Self::PipePipe => write!(f, "||"),
            Self::Bang => write!(f, "!"),
            Self::Equal => write!(f, "="),
            Self::PlusEqual => write!(f, "+="),
            Self::MinusEqual => write!(f, "-="),
            Self::StarEqual => write!(f, "*="),
            Self::SlashEqual => write!(f, "/="),
            Self::PercentEqual => write!(f, "%="),
            Self::Ampersand => write!(f, "&"),
            Self::AmpMut => write!(f, "&mut"),
            Self::Question => write!(f, "?"),
            Self::DotDot => write!(f, ".."),
            Self::DotDotEqual => write!(f, "..="),
            Self::FatArrow => write!(f, "=>"),
            Self::Arrow => write!(f, "->"),

            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::LBrace => write!(f, "{{"),
            Self::RBrace => write!(f, "}}"),
            Self::LBracket => write!(f, "["),
            Self::RBracket => write!(f, "]"),
            Self::Colon => write!(f, ":"),
            Self::Comma => write!(f, ","),
            Self::Dot => write!(f, "."),
            Self::Semicolon => write!(f, ";"),
            Self::At => write!(f, "@"),

            Self::Eof => write!(f, "EOF"),
        }
    }
}
