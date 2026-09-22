//! Lexical scope tree and lookup infrastructure.

use std::collections::HashMap;
use std::fmt;

use crate::symbol::SymbolId;

/// Unique identifier for a lexical scope in a [`ScopeTree`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(pub usize);

impl fmt::Display for ScopeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scope({})", self.0)
    }
}

/// The architectural kind of a lexical scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScopeKind {
    /// Root top-level module/file scope.
    Global,
    /// Function body and parameter scope.
    Function,
    /// Local nested block `{ ... }` scope.
    Block,
    /// Anonymous lambda parameter and body scope.
    Lambda,
    /// Pattern match arm scope.
    MatchArm,
}

/// A lexical scope node storing bindings in value and type namespaces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope {
    /// Unique index of this scope.
    pub id: ScopeId,
    /// Semantic kind of scope.
    pub kind: ScopeKind,
    /// Parent enclosing scope, or `None` for the root global scope.
    pub parent: Option<ScopeId>,
    /// Value bindings (variables, constants, parameters, functions, enums, structs).
    pub values: HashMap<String, SymbolId>,
    /// Type bindings (structs, enums, traits, type parameters, primitive types).
    pub types: HashMap<String, SymbolId>,
}

impl Scope {
    /// Creates a new `Scope`.
    pub fn new(id: ScopeId, kind: ScopeKind, parent: Option<ScopeId>) -> Self {
        Self {
            id,
            kind,
            parent,
            values: HashMap::new(),
            types: HashMap::new(),
        }
    }
}

/// Arena managing all lexical scopes and resolution hierarchies in a compilation unit.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScopeTree {
    scopes: Vec<Scope>,
}

impl ScopeTree {
    /// Creates an empty `ScopeTree`.
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    /// Allocates and records a new scope, returning its unique [`ScopeId`].
    pub fn alloc(&mut self, kind: ScopeKind, parent: Option<ScopeId>) -> ScopeId {
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope::new(id, kind, parent));
        id
    }

    /// Retrieves a scope reference by [`ScopeId`].
    pub fn get(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(id.0)
    }

    /// Retrieves a mutable scope reference by [`ScopeId`].
    pub fn get_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(id.0)
    }

    /// Defines a value symbol in the specified scope.
    ///
    /// # Errors
    /// Returns `Err(existing_symbol_id)` if a symbol with the same name is already
    /// declared in this immediate scope (duplicate declaration).
    pub fn define_value(
        &mut self,
        scope_id: ScopeId,
        name: impl Into<String>,
        symbol_id: SymbolId,
    ) -> Result<(), SymbolId> {
        let name_str = name.into();
        let scope = self
            .get_mut(scope_id)
            .expect("attempted to define symbol in invalid scope");

        if let Some(&prev) = scope.values.get(&name_str) {
            return Err(prev);
        }

        scope.values.insert(name_str, symbol_id);
        Ok(())
    }

    /// Defines a type symbol in the specified scope.
    ///
    /// # Errors
    /// Returns `Err(existing_symbol_id)` if a type symbol with the same name is already
    /// declared in this immediate scope.
    pub fn define_type(
        &mut self,
        scope_id: ScopeId,
        name: impl Into<String>,
        symbol_id: SymbolId,
    ) -> Result<(), SymbolId> {
        let name_str = name.into();
        let scope = self
            .get_mut(scope_id)
            .expect("attempted to define type in invalid scope");

        if let Some(&prev) = scope.types.get(&name_str) {
            return Err(prev);
        }

        scope.types.insert(name_str, symbol_id);
        Ok(())
    }

    /// Looks up a value symbol starting from `scope_id`, traversing parent scopes.
    pub fn lookup_value(&self, mut current: ScopeId, name: &str) -> Option<SymbolId> {
        loop {
            let scope = self.get(current)?;
            if let Some(&sym) = scope.values.get(name) {
                return Some(sym);
            }
            // Fall back to types if an enum or struct is referenced as a value (e.g. `Status.Active`)
            if let Some(&sym) = scope.types.get(name) {
                return Some(sym);
            }
            current = scope.parent?;
        }
    }

    /// Looks up a value symbol strictly within the specified scope (non-recursive).
    pub fn lookup_value_current(&self, scope_id: ScopeId, name: &str) -> Option<SymbolId> {
        let scope = self.get(scope_id)?;
        scope
            .values
            .get(name)
            .copied()
            .or_else(|| scope.types.get(name).copied())
    }

    /// Looks up a type symbol starting from `scope_id`, traversing parent scopes.
    pub fn lookup_type(&self, mut current: ScopeId, name: &str) -> Option<SymbolId> {
        loop {
            let scope = self.get(current)?;
            if let Some(&sym) = scope.types.get(name) {
                return Some(sym);
            }
            current = scope.parent?;
        }
    }

    /// Looks up a type symbol strictly within the specified scope (non-recursive).
    pub fn lookup_type_current(&self, scope_id: ScopeId, name: &str) -> Option<SymbolId> {
        self.get(scope_id)?.types.get(name).copied()
    }

    /// Returns the total number of scopes allocated.
    pub fn len(&self) -> usize {
        self.scopes.len()
    }

    /// Returns `true` if no scopes have been allocated.
    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }
}
