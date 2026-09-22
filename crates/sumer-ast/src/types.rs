//! Type syntax representations in the SUMER AST.
//!
//! Represents purely syntactic type expressions as written in source code,
//! completely decoupled from the semantic type system.

use sumer_span::Span;

use crate::expression::Expr;
use crate::identifier::Identifier;

/// A path to a named type (e.g. `User`, `std.collections.List`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypePath {
    /// Segments in the type path separated by `.`.
    pub segments: Vec<Identifier>,
    /// Span covering the entire path.
    pub span: Span,
}

impl TypePath {
    /// Creates a new `TypePath`.
    pub fn new(segments: Vec<Identifier>, span: Span) -> Self {
        Self { segments, span }
    }

    /// Creates a single-segment `TypePath` from an identifier.
    pub fn single(ident: Identifier) -> Self {
        let span = ident.span;
        Self {
            segments: vec![ident],
            span,
        }
    }
}

/// The specific syntactic form of a type expression.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeKind {
    /// Named type (e.g. `Int`, `String`, `User`).
    Named(TypePath),

    /// Generic instantiated type (e.g. `Result<User, Error>`, `List<Int>`).
    Generic {
        name: TypePath,
        arguments: Vec<Type>,
    },

    /// Immutable reference type (e.g. `&User`).
    Reference { inner: Box<Type> },

    /// Mutable reference type (e.g. `&mut User`).
    MutableReference { inner: Box<Type> },

    /// Optional type (e.g. `User?` or `?User`).
    Optional { inner: Box<Type> },

    /// Function signature type (e.g. `fn(Int, Int) -> Bool`).
    Function {
        parameters: Vec<Type>,
        return_type: Box<Type>,
    },

    /// Tuple type (e.g. `(Int, String)`).
    Tuple { elements: Vec<Type> },

    /// Array type with optional compile-time length expression (e.g. `[Int]`, `[Int; 4]`).
    Array {
        element: Box<Type>,
        size: Option<Box<Expr>>,
    },
}

/// A type annotation in the AST with source span.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Type {
    /// Kind of type syntax.
    pub kind: TypeKind,
    /// Span covering the entire type expression.
    pub span: Span,
}

impl Type {
    /// Creates a new `Type`.
    pub fn new(kind: TypeKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Helper to construct a named type.
    pub fn named(path: TypePath, span: Span) -> Self {
        Self::new(TypeKind::Named(path), span)
    }

    /// Helper to construct an immutable reference type.
    pub fn reference(inner: Type, span: Span) -> Self {
        Self::new(
            TypeKind::Reference {
                inner: Box::new(inner),
            },
            span,
        )
    }

    /// Helper to construct a mutable reference type.
    pub fn mut_reference(inner: Type, span: Span) -> Self {
        Self::new(
            TypeKind::MutableReference {
                inner: Box::new(inner),
            },
            span,
        )
    }

    /// Helper to construct an optional type.
    pub fn optional(inner: Type, span: Span) -> Self {
        Self::new(
            TypeKind::Optional {
                inner: Box::new(inner),
            },
            span,
        )
    }

    /// Helper to construct a generic type.
    pub fn generic(name: TypePath, arguments: Vec<Type>, span: Span) -> Self {
        Self::new(TypeKind::Generic { name, arguments }, span)
    }

    /// Helper to construct a function type.
    pub fn function(parameters: Vec<Type>, return_type: Type, span: Span) -> Self {
        Self::new(
            TypeKind::Function {
                parameters,
                return_type: Box::new(return_type),
            },
            span,
        )
    }

    /// Helper to construct a tuple type.
    pub fn tuple(elements: Vec<Type>, span: Span) -> Self {
        Self::new(TypeKind::Tuple { elements }, span)
    }

    /// Helper to construct an array type.
    pub fn array(element: Type, size: Option<Expr>, span: Span) -> Self {
        Self::new(
            TypeKind::Array {
                element: Box::new(element),
                size: size.map(Box::new),
            },
            span,
        )
    }
}
