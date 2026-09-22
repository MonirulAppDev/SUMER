//! Pattern matching AST nodes.

use sumer_span::Span;

use crate::expression::Literal;
use crate::identifier::Identifier;
use crate::types::TypePath;

/// A field matched inside a struct pattern (e.g. `name` or `name: renamed`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FieldPattern {
    /// Name of the field.
    pub name: Identifier,
    /// Sub-pattern if renamed (e.g. `Some(pattern)` in `User { name: n }`).
    pub pattern: Option<Pattern>,
    /// Span covering this field pattern.
    pub span: Span,
}

impl FieldPattern {
    /// Creates a new `FieldPattern`.
    pub fn new(name: Identifier, pattern: Option<Pattern>, span: Span) -> Self {
        Self {
            name,
            pattern,
            span,
        }
    }
}

/// The specific pattern category.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PatternKind {
    /// Wildcard pattern `_`.
    Wildcard,

    /// Identifier pattern binding (e.g. `x`, `user`).
    Identifier(Identifier),

    /// Constant literal pattern (e.g. `10`, `"admin"`, `true`).
    Literal(Literal),

    /// Enum variant pattern (e.g. `Payment.Cash`, `Payment.Card(number)`).
    Enum {
        path: TypePath,
        variant: Identifier,
        data: Option<Vec<Pattern>>,
    },

    /// Destructuring struct pattern (e.g. `User { name }`).
    Struct {
        path: TypePath,
        fields: Vec<FieldPattern>,
    },

    /// Tuple destructuring pattern (e.g. `(a, b)`).
    Tuple(Vec<Pattern>),
}

/// A pattern expression in match arms or let bindings.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Pattern {
    /// The kind of pattern.
    pub kind: PatternKind,
    /// Source span covering the pattern.
    pub span: Span,
}

impl Pattern {
    /// Creates a new `Pattern`.
    pub fn new(kind: PatternKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Helper to construct a wildcard pattern `_`.
    pub fn wildcard(span: Span) -> Self {
        Self::new(PatternKind::Wildcard, span)
    }

    /// Helper to construct an identifier pattern.
    pub fn identifier(ident: Identifier) -> Self {
        let span = ident.span;
        Self::new(PatternKind::Identifier(ident), span)
    }

    /// Helper to construct a literal pattern.
    pub fn literal(lit: Literal, span: Span) -> Self {
        Self::new(PatternKind::Literal(lit), span)
    }

    /// Helper to construct an enum variant pattern.
    pub fn enum_variant(
        path: TypePath,
        variant: Identifier,
        data: Option<Vec<Pattern>>,
        span: Span,
    ) -> Self {
        Self::new(
            PatternKind::Enum {
                path,
                variant,
                data,
            },
            span,
        )
    }

    /// Helper to construct a struct pattern.
    pub fn struct_pattern(path: TypePath, fields: Vec<FieldPattern>, span: Span) -> Self {
        Self::new(PatternKind::Struct { path, fields }, span)
    }

    /// Helper to construct a tuple pattern.
    pub fn tuple(elements: Vec<Pattern>, span: Span) -> Self {
        Self::new(PatternKind::Tuple(elements), span)
    }
}
