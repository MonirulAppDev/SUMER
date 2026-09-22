//! Identifier AST node.

use std::fmt;
use sumer_span::Span;

/// A named identifier token with its source location.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier {
    /// Name of the identifier in source code.
    pub name: String,
    /// Source span covering this identifier.
    pub span: Span,
}

impl Identifier {
    /// Creates a new `Identifier`.
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }

    /// Returns a string slice of the identifier's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the source span of the identifier.
    pub const fn span(&self) -> Span {
        self.span
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
