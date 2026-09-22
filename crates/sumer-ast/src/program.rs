//! Root Program AST node representing an entire SUMER source file.

use sumer_span::Span;

use crate::attribute::Attribute;
use crate::declaration::Declaration;

/// Represents an entire parsed SUMER source file (`.sm`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Program {
    /// File-level attributes/annotations.
    pub attributes: Vec<Attribute>,
    /// Top-level declarations.
    pub declarations: Vec<Declaration>,
    /// Source span covering the whole source file.
    pub span: Span,
}

impl Program {
    /// Creates a new `Program`.
    pub fn new(attributes: Vec<Attribute>, declarations: Vec<Declaration>, span: Span) -> Self {
        Self {
            attributes,
            declarations,
            span,
        }
    }

    /// Creates an empty `Program`.
    pub fn empty(span: Span) -> Self {
        Self {
            attributes: Vec::new(),
            declarations: Vec::new(),
            span,
        }
    }
}
