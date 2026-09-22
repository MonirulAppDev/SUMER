//! Attribute AST nodes (e.g. `@derive(Debug)`).

use sumer_span::Span;

use crate::identifier::Identifier;

/// An argument passed inside an attribute.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AttributeArg {
    /// An identifier argument (e.g. `Debug` in `@derive(Debug)`).
    Identifier(Identifier),
    /// A literal string argument (e.g. `"windows"`).
    String(String),
}

/// An attribute decorating declarations or items in SUMER.
///
/// Examples:
/// ```sumer
/// @derive(Debug)
/// @inline
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Attribute {
    /// The name of the attribute (e.g. `derive`).
    pub name: Identifier,
    /// Arguments provided to the attribute in parentheses.
    pub arguments: Vec<AttributeArg>,
    /// Source span covering the attribute including `@`.
    pub span: Span,
}

impl Attribute {
    /// Creates a new `Attribute`.
    pub fn new(name: Identifier, arguments: Vec<AttributeArg>, span: Span) -> Self {
        Self {
            name,
            arguments,
            span,
        }
    }

    /// Helper to create a simple attribute with identifier arguments.
    pub fn simple(name: Identifier, idents: Vec<Identifier>, span: Span) -> Self {
        let arguments = idents.into_iter().map(AttributeArg::Identifier).collect();
        Self {
            name,
            arguments,
            span,
        }
    }
}
