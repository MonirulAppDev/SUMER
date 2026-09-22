//! Declaration AST nodes (functions, structs, enums, traits, impls, variables, constants, imports).

use sumer_span::Span;

use crate::attribute::Attribute;
use crate::expression::Expr;
use crate::generics::{GenericParams, TypeBound};
use crate::identifier::Identifier;
use crate::statement::Block;
use crate::types::Type;

/// Item visibility specification in SUMER source code.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Visibility {
    /// Private visibility (default in SUMER).
    #[default]
    Private,
    /// Public visibility marked with `pub`.
    Public,
}

impl Visibility {
    /// Returns `true` if public.
    pub fn is_public(&self) -> bool {
        matches!(self, Self::Public)
    }
}

/// A formal parameter in a function declaration or signature.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Parameter {
    /// Parameter identifier name.
    pub name: Identifier,
    /// Explicit type of the parameter.
    pub param_type: Type,
    /// Optional default argument expression (e.g. `fn greet(name: String = "World")`).
    pub default_value: Option<Expr>,
    /// Span covering the parameter definition.
    pub span: Span,
}

impl Parameter {
    /// Creates a new `Parameter`.
    pub fn new(
        name: Identifier,
        param_type: Type,
        default_value: Option<Expr>,
        span: Span,
    ) -> Self {
        Self {
            name,
            param_type,
            default_value,
            span,
        }
    }
}

/// A function declaration with a complete body.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionDecl {
    /// Name of the function.
    pub name: Identifier,
    /// Visibility (`pub` or private).
    pub visibility: Visibility,
    /// Indicates whether the function is declared `async`.
    pub is_async: bool,
    /// Generic type parameters.
    pub generics: GenericParams,
    /// Formal parameters.
    pub parameters: Vec<Parameter>,
    /// Optional return type annotation.
    pub return_type: Option<Type>,
    /// Body block containing the function statements.
    pub body: Block,
    /// Preceding decorators/attributes.
    pub attributes: Vec<Attribute>,
    /// Span covering the entire function declaration.
    pub span: Span,
}

impl FunctionDecl {
    /// Creates a new `FunctionDecl`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: Identifier,
        visibility: Visibility,
        is_async: bool,
        generics: GenericParams,
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        body: Block,
        attributes: Vec<Attribute>,
        span: Span,
    ) -> Self {
        Self {
            name,
            visibility,
            is_async,
            generics,
            parameters,
            return_type,
            body,
            attributes,
            span,
        }
    }
}

/// A field member declared within a struct.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FieldDecl {
    /// Field identifier.
    pub name: Identifier,
    /// Visibility of the field.
    pub visibility: Visibility,
    /// Field type annotation.
    pub field_type: Type,
    /// Optional default value expression.
    pub default_value: Option<Expr>,
    /// Preceding attributes.
    pub attributes: Vec<Attribute>,
    /// Span covering this field declaration.
    pub span: Span,
}

impl FieldDecl {
    /// Creates a new `FieldDecl`.
    pub fn new(
        name: Identifier,
        visibility: Visibility,
        field_type: Type,
        default_value: Option<Expr>,
        attributes: Vec<Attribute>,
        span: Span,
    ) -> Self {
        Self {
            name,
            visibility,
            field_type,
            default_value,
            attributes,
            span,
        }
    }
}

/// A member inside a struct declaration (field or method).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StructMember {
    /// A data field.
    Field(FieldDecl),
    /// A method defined within the struct body.
    Method(FunctionDecl),
}

/// A struct data type declaration.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructDecl {
    /// Struct name identifier.
    pub name: Identifier,
    /// Visibility.
    pub visibility: Visibility,
    /// Generic type parameters.
    pub generics: GenericParams,
    /// Struct members (fields and methods).
    pub members: Vec<StructMember>,
    /// Preceding attributes.
    pub attributes: Vec<Attribute>,
    /// Span covering the struct declaration.
    pub span: Span,
}

impl StructDecl {
    /// Creates a new `StructDecl`.
    pub fn new(
        name: Identifier,
        visibility: Visibility,
        generics: GenericParams,
        members: Vec<StructMember>,
        attributes: Vec<Attribute>,
        span: Span,
    ) -> Self {
        Self {
            name,
            visibility,
            generics,
            members,
            attributes,
            span,
        }
    }
}

/// The data layout carried by an enum variant.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VariantData {
    /// Unit variant with no data (e.g. `Cash`, `Active`).
    Unit,
    /// Positional tuple-style fields (e.g. `Result(String)`).
    Tuple(Vec<Type>),
    /// Named fields (e.g. `Card(number: String)`).
    Struct(Vec<FieldDecl>),
}

/// An individual variant inside an enum declaration.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EnumVariant {
    /// Variant name identifier.
    pub name: Identifier,
    /// Payload data carried by this variant.
    pub data: VariantData,
    /// Span covering this variant.
    pub span: Span,
}

impl EnumVariant {
    /// Creates a new `EnumVariant`.
    pub fn new(name: Identifier, data: VariantData, span: Span) -> Self {
        Self { name, data, span }
    }
}

/// An algebraic enum type declaration.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EnumDecl {
    /// Enum name identifier.
    pub name: Identifier,
    /// Visibility.
    pub visibility: Visibility,
    /// Generic type parameters.
    pub generics: GenericParams,
    /// Variants defined inside the enum.
    pub variants: Vec<EnumVariant>,
    /// Preceding attributes.
    pub attributes: Vec<Attribute>,
    /// Span covering the enum declaration.
    pub span: Span,
}

impl EnumDecl {
    /// Creates a new `EnumDecl`.
    pub fn new(
        name: Identifier,
        visibility: Visibility,
        generics: GenericParams,
        variants: Vec<EnumVariant>,
        attributes: Vec<Attribute>,
        span: Span,
    ) -> Self {
        Self {
            name,
            visibility,
            generics,
            variants,
            attributes,
            span,
        }
    }
}

/// An abstract function signature declared in a trait.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionSignature {
    /// Function name identifier.
    pub name: Identifier,
    /// Indicates whether the function is declared `async`.
    pub is_async: bool,
    /// Generic type parameters.
    pub generics: GenericParams,
    /// Formal parameters.
    pub parameters: Vec<Parameter>,
    /// Optional return type annotation.
    pub return_type: Option<Type>,
    /// Span covering the signature.
    pub span: Span,
}

impl FunctionSignature {
    /// Creates a new `FunctionSignature`.
    pub fn new(
        name: Identifier,
        is_async: bool,
        generics: GenericParams,
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        span: Span,
    ) -> Self {
        Self {
            name,
            is_async,
            generics,
            parameters,
            return_type,
            span,
        }
    }
}

/// A member declared within a trait.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TraitMember {
    /// Abstract function signature (no body).
    FunctionSignature(FunctionSignature),
    /// Default function implementation.
    Function(FunctionDecl),
}

/// A trait interface declaration.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitDecl {
    /// Trait name identifier.
    pub name: Identifier,
    /// Visibility.
    pub visibility: Visibility,
    /// Generic type parameters.
    pub generics: GenericParams,
    /// Supertrait bounds (e.g. `trait Printable: Debug + Clone`).
    pub bounds: Vec<TypeBound>,
    /// Trait members (signatures and default implementations).
    pub members: Vec<TraitMember>,
    /// Preceding attributes.
    pub attributes: Vec<Attribute>,
    /// Span covering the trait declaration.
    pub span: Span,
}

impl TraitDecl {
    /// Creates a new `TraitDecl`.
    pub fn new(
        name: Identifier,
        visibility: Visibility,
        generics: GenericParams,
        bounds: Vec<TypeBound>,
        members: Vec<TraitMember>,
        attributes: Vec<Attribute>,
        span: Span,
    ) -> Self {
        Self {
            name,
            visibility,
            generics,
            bounds,
            members,
            attributes,
            span,
        }
    }
}

/// An implementation block (inherent or trait implementation).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplDecl {
    /// Generic type parameters.
    pub generics: GenericParams,
    /// Trait type if this is a trait implementation (`impl Printable for User`).
    pub trait_type: Option<Type>,
    /// Target type being implemented.
    pub target_type: Type,
    /// Implemented methods.
    pub members: Vec<FunctionDecl>,
    /// Preceding attributes.
    pub attributes: Vec<Attribute>,
    /// Span covering the impl block.
    pub span: Span,
}

impl ImplDecl {
    /// Creates a new `ImplDecl`.
    pub fn new(
        generics: GenericParams,
        trait_type: Option<Type>,
        target_type: Type,
        members: Vec<FunctionDecl>,
        attributes: Vec<Attribute>,
        span: Span,
    ) -> Self {
        Self {
            generics,
            trait_type,
            target_type,
            members,
            attributes,
            span,
        }
    }
}

/// A variable binding declaration (`let` or `var`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VariableDecl {
    /// Bound variable name identifier.
    pub name: Identifier,
    /// Mutability flag (`true` for `var`, `false` for `let`).
    pub is_mutable: bool,
    /// Optional explicit type annotation.
    pub explicit_type: Option<Type>,
    /// Initializer value expression.
    pub initializer: Expr,
    /// Visibility if declared at module level.
    pub visibility: Visibility,
    /// Span covering the variable declaration.
    pub span: Span,
}

impl VariableDecl {
    /// Creates a new `VariableDecl`.
    pub fn new(
        name: Identifier,
        is_mutable: bool,
        explicit_type: Option<Type>,
        initializer: Expr,
        visibility: Visibility,
        span: Span,
    ) -> Self {
        Self {
            name,
            is_mutable,
            explicit_type,
            initializer,
            visibility,
            span,
        }
    }
}

/// A constant item declaration (`const MAX_USERS = 1000`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConstantDecl {
    /// Constant identifier name.
    pub name: Identifier,
    /// Optional explicit type annotation.
    pub explicit_type: Option<Type>,
    /// Constant value expression.
    pub value: Expr,
    /// Visibility.
    pub visibility: Visibility,
    /// Preceding attributes.
    pub attributes: Vec<Attribute>,
    /// Span covering the constant declaration.
    pub span: Span,
}

impl ConstantDecl {
    /// Creates a new `ConstantDecl`.
    pub fn new(
        name: Identifier,
        explicit_type: Option<Type>,
        value: Expr,
        visibility: Visibility,
        attributes: Vec<Attribute>,
        span: Span,
    ) -> Self {
        Self {
            name,
            explicit_type,
            value,
            visibility,
            attributes,
            span,
        }
    }
}

/// A module path in an import declaration (e.g. `user.User`, `std.io`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModulePath {
    /// Segments of the module path.
    pub segments: Vec<Identifier>,
    /// Span covering the path.
    pub span: Span,
}

impl ModulePath {
    /// Creates a new `ModulePath`.
    pub fn new(segments: Vec<Identifier>, span: Span) -> Self {
        Self { segments, span }
    }
}

/// An import declaration (e.g. `import user.User`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImportDecl {
    /// Imported module path.
    pub path: ModulePath,
    /// Visibility (e.g. `pub import`).
    pub visibility: Visibility,
    /// Span covering the import declaration.
    pub span: Span,
}

impl ImportDecl {
    /// Creates a new `ImportDecl`.
    pub fn new(path: ModulePath, visibility: Visibility, span: Span) -> Self {
        Self {
            path,
            visibility,
            span,
        }
    }
}

/// Top-level declaration variants supported in a SUMER program.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Declaration {
    /// Function declaration.
    Function(FunctionDecl),
    /// Struct declaration.
    Struct(StructDecl),
    /// Enum declaration.
    Enum(EnumDecl),
    /// Trait declaration.
    Trait(TraitDecl),
    /// Implementation block.
    Impl(ImplDecl),
    /// Variable binding.
    Variable(VariableDecl),
    /// Constant item.
    Constant(ConstantDecl),
    /// Import declaration.
    Import(ImportDecl),
}

impl Declaration {
    /// Returns the source span covering this declaration.
    pub fn span(&self) -> Span {
        match self {
            Self::Function(d) => d.span,
            Self::Struct(d) => d.span,
            Self::Enum(d) => d.span,
            Self::Trait(d) => d.span,
            Self::Impl(d) => d.span,
            Self::Variable(d) => d.span,
            Self::Constant(d) => d.span,
            Self::Import(d) => d.span,
        }
    }
}
