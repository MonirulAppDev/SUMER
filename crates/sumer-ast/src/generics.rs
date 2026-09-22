//! Generic parameter and trait bound AST nodes.

use sumer_span::Span;

use crate::identifier::Identifier;

/// A trait bound applied to a generic parameter (e.g. `Comparable` in `<T: Comparable>`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeBound {
    /// The trait identifier bound.
    pub name: Identifier,
    /// Span covering this bound.
    pub span: Span,
}

impl TypeBound {
    /// Creates a new `TypeBound`.
    pub fn new(name: Identifier, span: Span) -> Self {
        Self { name, span }
    }
}

/// A single generic parameter declaration (e.g. `T: Clone + Debug`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GenericParam {
    /// Parameter identifier name (e.g. `T`).
    pub name: Identifier,
    /// Trait bounds on this parameter.
    pub bounds: Vec<TypeBound>,
    /// Span covering this parameter declaration.
    pub span: Span,
}

impl GenericParam {
    /// Creates a new `GenericParam`.
    pub fn new(name: Identifier, bounds: Vec<TypeBound>, span: Span) -> Self {
        Self { name, bounds, span }
    }
}

/// A list of generic parameters (e.g. `<T, U: Printable>`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct GenericParams {
    /// The declared generic parameters.
    pub params: Vec<GenericParam>,
    /// Span covering `<...>` enclosing the generics.
    pub span: Span,
}

impl GenericParams {
    /// Creates a new `GenericParams` container.
    pub fn new(params: Vec<GenericParam>, span: Span) -> Self {
        Self { params, span }
    }

    /// Returns `true` if no generic parameters are present.
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
}
