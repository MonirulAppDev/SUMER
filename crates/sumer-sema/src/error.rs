//! Semantic analysis errors and diagnostic generation.

use std::error::Error;
use std::fmt;
use sumer_diagnostics::Diagnostic;
use sumer_span::Span;

/// Errors detected during declaration collection, scope construction, name resolution, and type checking.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticError {
    /// An identifier was declared more than once in the same lexical scope.
    DuplicateDeclaration {
        /// Name of the duplicated identifier.
        name: String,
        /// Span of the new duplicate declaration.
        span: Span,
        /// Span of the original declaration.
        previous_span: Span,
    },
    /// An identifier reference could not be resolved in the lexical scope chain.
    UnresolvedIdentifier {
        /// Name of the missing identifier.
        name: String,
        /// Span of the identifier reference.
        span: Span,
    },
    /// A type reference could not be resolved against declared types or built-in primitives.
    UnresolvedType {
        /// Name of the unresolved type.
        name: String,
        /// Span of the type reference.
        span: Span,
    },
    /// A value or expression did not match the expected semantic type.
    TypeMismatch {
        /// Expected type name.
        expected: String,
        /// Actual inferred or found type name.
        found: String,
        /// Source span where the mismatch occurred.
        span: Span,
    },
    /// A binary operator cannot be applied to the operand types.
    CannotApplyBinary {
        /// Operator string (e.g. `+`, `-`).
        op: String,
        /// Left operand type name.
        left: String,
        /// Right operand type name.
        right: String,
        /// Source span covering the binary expression.
        span: Span,
    },
    /// A unary prefix operator cannot be applied to the operand type.
    CannotApplyUnary {
        /// Operator string (e.g. `!`, `-`).
        op: String,
        /// Operand type name.
        operand: String,
        /// Source span covering the unary expression.
        span: Span,
    },
    /// A function call argument does not match the parameter type.
    ArgumentTypeMismatch {
        /// Expected parameter type.
        expected: String,
        /// Actual argument type.
        found: String,
        /// Source span of the argument.
        span: Span,
    },
    /// The number of supplied arguments does not match function parameter requirements.
    ArgumentCountMismatch {
        /// Expected argument count.
        expected: usize,
        /// Number of arguments supplied.
        found: usize,
        /// Source span of the call expression.
        span: Span,
    },
    /// A return statement expression does not match the function return type annotation.
    ReturnTypeMismatch {
        /// Expected return type.
        expected: String,
        /// Found return expression type.
        found: String,
        /// Source span of the return expression.
        span: Span,
    },
    /// A condition in `if` or `while` is not `Bool`.
    ConditionTypeMismatch {
        /// Found condition type name.
        found: String,
        /// Source span of the condition.
        span: Span,
    },
    /// An attempt was made to call a non-callable value as a function.
    NotCallable {
        /// Type of the non-callable value.
        found: String,
        /// Source span of the callee.
        span: Span,
    },
    /// A trait reference in a bound or impl could not be resolved.
    UnknownTrait {
        /// Name of the missing trait.
        name: String,
        /// Span of the trait reference.
        span: Span,
    },
    /// A trait is implemented more than once for the same type.
    DuplicateTraitImpl {
        /// Name of the trait.
        trait_name: String,
        /// Name of the target type.
        target_type: String,
        /// Span of the duplicate implementation.
        span: Span,
        /// Span of the original implementation.
        previous_span: Span,
    },
    /// A generic type was instantiated with an incorrect number of type arguments.
    WrongGenericArgCount {
        /// Name of the generic type.
        name: String,
        /// Expected number of type arguments.
        expected: usize,
        /// Actual number of type arguments provided.
        found: usize,
        /// Span of the generic type instantiation.
        span: Span,
    },
    /// A concrete type does not satisfy a required trait bound.
    TraitBoundNotSatisfied {
        /// Concrete type name.
        ty: String,
        /// Name of the unsatisfied trait bound.
        trait_name: String,
        /// Span where the bound violation occurred.
        span: Span,
    },
}

impl SemanticError {
    /// Returns the primary source span associated with this error.
    pub fn span(&self) -> Span {
        match self {
            Self::DuplicateDeclaration { span, .. } => *span,
            Self::UnresolvedIdentifier { span, .. } => *span,
            Self::UnresolvedType { span, .. } => *span,
            Self::TypeMismatch { span, .. } => *span,
            Self::CannotApplyBinary { span, .. } => *span,
            Self::CannotApplyUnary { span, .. } => *span,
            Self::ArgumentTypeMismatch { span, .. } => *span,
            Self::ArgumentCountMismatch { span, .. } => *span,
            Self::ReturnTypeMismatch { span, .. } => *span,
            Self::ConditionTypeMismatch { span, .. } => *span,
            Self::NotCallable { span, .. } => *span,
            Self::UnknownTrait { span, .. } => *span,
            Self::DuplicateTraitImpl { span, .. } => *span,
            Self::WrongGenericArgCount { span, .. } => *span,
            Self::TraitBoundNotSatisfied { span, .. } => *span,
        }
    }

    /// Converts this semantic error into a structured compiler [`Diagnostic`].
    pub fn to_diagnostic(&self) -> Diagnostic {
        match self {
            Self::DuplicateDeclaration {
                name,
                span,
                previous_span,
            } => Diagnostic::error(format!("duplicate declaration `{name}`"))
                .with_primary(*span, "duplicate declaration")
                .with_secondary(*previous_span, "previous declaration is here")
                .with_note(format!("`{name}` has already been declared in this scope")),

            Self::UnresolvedIdentifier { name, span } => {
                Diagnostic::error(format!("cannot find `{name}` in this scope"))
                    .with_primary(*span, "not found in this scope")
            }

            Self::UnresolvedType { name, span } => {
                Diagnostic::error(format!("cannot find type `{name}` in this scope"))
                    .with_primary(*span, "type not found in this scope")
            }

            Self::TypeMismatch {
                expected,
                found,
                span,
            } => Diagnostic::error("type mismatch")
                .with_primary(*span, format!("expected `{expected}`, found `{found}`")),

            Self::CannotApplyBinary {
                op,
                left,
                right,
                span,
            } => Diagnostic::error(format!(
                "cannot apply binary operator `{op}` to types `{left}` and `{right}`"
            ))
            .with_primary(
                *span,
                format!("cannot apply `{op}` to `{left}` and `{right}`"),
            ),

            Self::CannotApplyUnary { op, operand, span } => Diagnostic::error(format!(
                "cannot apply unary operator `{op}` to type `{operand}`"
            ))
            .with_primary(*span, format!("cannot apply `{op}` to `{operand}`")),

            Self::ArgumentTypeMismatch {
                expected,
                found,
                span,
            } => Diagnostic::error("argument type mismatch")
                .with_primary(*span, format!("expected `{expected}`, found `{found}`")),

            Self::ArgumentCountMismatch {
                expected,
                found,
                span,
            } => Diagnostic::error(format!(
                "this function takes {expected} arguments but {found} were supplied"
            ))
            .with_primary(
                *span,
                format!("expected {expected} arguments, found {found}"),
            ),

            Self::ReturnTypeMismatch {
                expected,
                found,
                span,
            } => Diagnostic::error("return type mismatch")
                .with_primary(*span, format!("expected `{expected}`, found `{found}`")),

            Self::ConditionTypeMismatch { found, span } => {
                Diagnostic::error("mismatched type in condition")
                    .with_primary(*span, format!("expected `Bool`, found `{found}`"))
            }

            Self::NotCallable { found, span } => {
                Diagnostic::error(format!("expected function, found `{found}`"))
                    .with_primary(*span, "not a function")
            }

            Self::UnknownTrait { name, span } => Diagnostic::error(format!("unknown trait `{name}`"))
                .with_primary(*span, "trait not found in this scope"),

            Self::DuplicateTraitImpl {
                trait_name,
                target_type,
                span,
                previous_span,
            } => Diagnostic::error(format!(
                "duplicate implementation of trait `{trait_name}` for `{target_type}`"
            ))
            .with_primary(*span, "duplicate implementation")
            .with_secondary(*previous_span, "previous implementation is here"),

            Self::WrongGenericArgCount {
                name,
                expected,
                found,
                span,
            } => Diagnostic::error(format!(
                "wrong number of generic arguments for `{name}`: expected {expected}, found {found}"
            ))
            .with_primary(
                *span,
                format!(
                    "expected {expected} generic argument{}, found {found}",
                    if *expected == 1 { "" } else { "s" }
                ),
            ),

            Self::TraitBoundNotSatisfied {
                ty,
                trait_name,
                span,
            } => Diagnostic::error(format!(
                "the trait bound `{ty}: {trait_name}` is not satisfied"
            ))
            .with_primary(
                *span,
                format!("trait `{trait_name}` is not implemented for `{ty}`"),
            ),
        }
    }
}

impl From<&SemanticError> for Diagnostic {
    fn from(err: &SemanticError) -> Self {
        err.to_diagnostic()
    }
}

impl From<SemanticError> for Diagnostic {
    fn from(err: SemanticError) -> Self {
        err.to_diagnostic()
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateDeclaration { name, .. } => {
                write!(f, "duplicate declaration `{name}`")
            }
            Self::UnresolvedIdentifier { name, .. } => {
                write!(f, "cannot find `{name}` in this scope")
            }
            Self::UnresolvedType { name, .. } => {
                write!(f, "cannot find type `{name}` in this scope")
            }
            Self::TypeMismatch {
                expected, found, ..
            } => write!(f, "type mismatch: expected `{expected}`, found `{found}`"),
            Self::CannotApplyBinary {
                op, left, right, ..
            } => write!(
                f,
                "cannot apply binary operator `{op}` to types `{left}` and `{right}`"
            ),
            Self::CannotApplyUnary { op, operand, .. } => {
                write!(f, "cannot apply unary operator `{op}` to type `{operand}`")
            }
            Self::ArgumentTypeMismatch {
                expected, found, ..
            } => write!(
                f,
                "argument type mismatch: expected `{expected}`, found `{found}`"
            ),
            Self::ArgumentCountMismatch {
                expected, found, ..
            } => write!(
                f,
                "this function takes {expected} arguments but {found} were supplied"
            ),
            Self::ReturnTypeMismatch {
                expected, found, ..
            } => write!(
                f,
                "return type mismatch: expected `{expected}`, found `{found}`"
            ),
            Self::ConditionTypeMismatch { found, .. } => {
                write!(f, "condition must be `Bool`, found `{found}`")
            }
            Self::NotCallable { found, .. } => write!(f, "expected function, found `{found}`"),
            Self::UnknownTrait { name, .. } => write!(f, "unknown trait `{name}`"),
            Self::DuplicateTraitImpl {
                trait_name,
                target_type,
                ..
            } => write!(
                f,
                "duplicate implementation of trait `{trait_name}` for `{target_type}`"
            ),
            Self::WrongGenericArgCount {
                name,
                expected,
                found,
                ..
            } => write!(
                f,
                "wrong number of generic arguments for `{name}`: expected {expected}, found {found}"
            ),
            Self::TraitBoundNotSatisfied {
                ty,
                trait_name,
                ..
            } => write!(f, "the trait bound `{ty}: {trait_name}` is not satisfied"),
        }
    }
}

impl Error for SemanticError {}
