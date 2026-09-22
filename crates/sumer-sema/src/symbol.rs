//! Symbol definitions and symbol table for semantic analysis.

use std::fmt;
use sumer_ast::Visibility;
use sumer_span::Span;

/// Unique identifier for a symbol allocated in a [`SymbolTable`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(pub usize);

impl fmt::Display for SymbolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sym({})", self.0)
    }
}

/// The semantic classification of a declared symbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    /// Function or method declaration.
    Function,
    /// Struct type declaration.
    Struct,
    /// Enum type declaration.
    Enum,
    /// Trait interface declaration.
    Trait,
    /// Local or module-level variable binding.
    Variable,
    /// Constant item declaration.
    Constant,
    /// Formal function parameter.
    Parameter,
    /// Generic type parameter (e.g. `T`).
    TypeParam,
    /// Primitive built-in type (e.g. `Int`, `String`).
    BuiltinType,
}

impl SymbolKind {
    /// Returns `true` if this symbol kind represents a type definition.
    pub fn is_type(&self) -> bool {
        matches!(
            self,
            Self::Struct | Self::Enum | Self::Trait | Self::TypeParam | Self::BuiltinType
        )
    }

    /// Returns `true` if this symbol kind represents a callable function.
    pub fn is_function(&self) -> bool {
        matches!(self, Self::Function)
    }
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Function => write!(f, "function"),
            Self::Struct => write!(f, "struct"),
            Self::Enum => write!(f, "enum"),
            Self::Trait => write!(f, "trait"),
            Self::Variable => write!(f, "variable"),
            Self::Constant => write!(f, "constant"),
            Self::Parameter => write!(f, "parameter"),
            Self::TypeParam => write!(f, "type parameter"),
            Self::BuiltinType => write!(f, "built-in type"),
        }
    }
}

/// A resolved or declared symbol in the SUMER semantic model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Symbol {
    /// Unique index of this symbol in the [`SymbolTable`].
    pub id: SymbolId,
    /// Name identifier of the symbol.
    pub name: String,
    /// Symbol variant kind.
    pub kind: SymbolKind,
    /// Source span where the symbol was declared.
    pub span: Span,
    /// Declared visibility.
    pub visibility: Visibility,
}

impl Symbol {
    /// Creates a new `Symbol`.
    pub fn new(
        id: SymbolId,
        name: impl Into<String>,
        kind: SymbolKind,
        span: Span,
        visibility: Visibility,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            span,
            visibility,
        }
    }
}

/// Arena-based storage for all allocated symbols in a compilation unit.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
}

impl SymbolTable {
    /// Creates an empty `SymbolTable`.
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
        }
    }

    /// Allocates and records a new symbol, returning its unique [`SymbolId`].
    pub fn alloc(
        &mut self,
        name: impl Into<String>,
        kind: SymbolKind,
        span: Span,
        visibility: Visibility,
    ) -> SymbolId {
        let id = SymbolId(self.symbols.len());
        self.symbols
            .push(Symbol::new(id, name, kind, span, visibility));
        id
    }

    /// Retrieves a symbol by its [`SymbolId`].
    pub fn get(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id.0)
    }

    /// Retrieves a mutable reference to a symbol by its [`SymbolId`].
    pub fn get_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
        self.symbols.get_mut(id.0)
    }

    /// Returns the total number of allocated symbols.
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Returns `true` if no symbols have been allocated.
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Returns an iterator over all allocated symbols.
    pub fn iter(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.iter()
    }
}
