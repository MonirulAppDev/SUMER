//! Semantic trait model, trait definitions, bounds, and implementation registry.

use std::collections::HashMap;
use std::fmt;

use sumer_span::Span;
use sumer_types::{GenericParamId, TypeId};

/// Unique 32-bit identifier for a declared semantic trait.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TraitId(pub u32);

impl fmt::Display for TraitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TraitId({})", self.0)
    }
}

/// A semantic trait bound applied to a generic parameter (e.g. `Comparable` in `T: Comparable`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraitBoundDef {
    /// Trait identifier name.
    pub trait_name: String,
    /// Resolved trait ID, or `None` if unresolved during error recovery.
    pub trait_id: Option<TraitId>,
    /// Source span covering the bound annotation.
    pub span: Span,
}

impl TraitBoundDef {
    /// Creates a new `TraitBoundDef`.
    pub fn new(trait_name: impl Into<String>, trait_id: Option<TraitId>, span: Span) -> Self {
        Self {
            trait_name: trait_name.into(),
            trait_id,
            span,
        }
    }
}

/// Semantic metadata for a declared generic parameter with its bounds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericParamDef {
    /// Stable parameter identity.
    pub id: GenericParamId,
    /// Name identifier (e.g. `"T"`).
    pub name: String,
    /// Trait bounds required for this parameter.
    pub bounds: Vec<TraitBoundDef>,
    /// Declaration span.
    pub span: Span,
}

impl GenericParamDef {
    /// Creates a new `GenericParamDef`.
    pub fn new(
        id: GenericParamId,
        name: impl Into<String>,
        bounds: Vec<TraitBoundDef>,
        span: Span,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            bounds,
            span,
        }
    }
}

/// Semantic definition of a method within a trait.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraitMethodDef {
    /// Method name identifier.
    pub name: String,
    /// Method generic type parameters.
    pub generic_params: Vec<GenericParamDef>,
    /// Formal parameter names and resolved semantic types.
    pub parameters: Vec<(String, TypeId)>,
    /// Method return type.
    pub return_type: TypeId,
    /// Indicates whether this method provides a default implementation.
    pub has_default: bool,
    /// Source span of the method declaration.
    pub span: Span,
}

impl TraitMethodDef {
    /// Creates a new `TraitMethodDef`.
    pub fn new(
        name: impl Into<String>,
        generic_params: Vec<GenericParamDef>,
        parameters: Vec<(String, TypeId)>,
        return_type: TypeId,
        has_default: bool,
        span: Span,
    ) -> Self {
        Self {
            name: name.into(),
            generic_params,
            parameters,
            return_type,
            has_default,
            span,
        }
    }
}

/// The semantic representation of a trait declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraitDef {
    /// Unique trait identifier.
    pub id: TraitId,
    /// Name of the trait.
    pub name: String,
    /// Generic type parameters declared on the trait.
    pub generic_params: Vec<GenericParamDef>,
    /// Methods defined within this trait.
    pub methods: Vec<TraitMethodDef>,
    /// Supertrait bounds.
    pub bounds: Vec<TraitBoundDef>,
    /// Source span covering the trait declaration.
    pub span: Span,
}

impl TraitDef {
    /// Creates a new `TraitDef`.
    pub fn new(
        id: TraitId,
        name: impl Into<String>,
        generic_params: Vec<GenericParamDef>,
        methods: Vec<TraitMethodDef>,
        bounds: Vec<TraitBoundDef>,
        span: Span,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            generic_params,
            methods,
            bounds,
            span,
        }
    }
}

/// Semantic metadata for a trait implementation block (`impl Trait for Type`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraitImplDef {
    /// Implemented trait identifier.
    pub trait_id: TraitId,
    /// Implemented trait name.
    pub trait_name: String,
    /// Concrete or applied target type implementing the trait.
    pub target_type: TypeId,
    /// Generic parameters on the `impl` block (e.g. `impl<T: Printable> Printable for Box<T>`).
    pub generic_params: Vec<GenericParamDef>,
    /// Method names provided in this implementation block.
    pub methods: Vec<String>,
    /// Source span covering the implementation block.
    pub span: Span,
}

impl TraitImplDef {
    /// Creates a new `TraitImplDef`.
    pub fn new(
        trait_id: TraitId,
        trait_name: impl Into<String>,
        target_type: TypeId,
        generic_params: Vec<GenericParamDef>,
        methods: Vec<String>,
        span: Span,
    ) -> Self {
        Self {
            trait_id,
            trait_name: trait_name.into(),
            target_type,
            generic_params,
            methods,
            span,
        }
    }
}

/// Semantic metadata for an inherent implementation block (`impl Type`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InherentImplDef {
    /// Target type being extended.
    pub target_type: TypeId,
    /// Generic parameters on the `impl` block.
    pub generic_params: Vec<GenericParamDef>,
    /// Method names provided in this inherent block.
    pub methods: Vec<String>,
    /// Source span covering the inherent implementation block.
    pub span: Span,
}

impl InherentImplDef {
    /// Creates a new `InherentImplDef`.
    pub fn new(
        target_type: TypeId,
        generic_params: Vec<GenericParamDef>,
        methods: Vec<String>,
        span: Span,
    ) -> Self {
        Self {
            target_type,
            generic_params,
            methods,
            span,
        }
    }
}

/// Central registry storing declared traits and implementations.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TraitRegistry {
    traits: HashMap<TraitId, TraitDef>,
    trait_names: HashMap<String, TraitId>,
    implementations: Vec<TraitImplDef>,
    inherent_impls: Vec<InherentImplDef>,
}

impl TraitRegistry {
    /// Creates an empty `TraitRegistry`.
    pub fn new() -> Self {
        Self {
            traits: HashMap::new(),
            trait_names: HashMap::new(),
            implementations: Vec::new(),
            inherent_impls: Vec::new(),
        }
    }

    /// Allocates a new [`TraitId`] for a new trait declaration.
    pub fn next_id(&self) -> TraitId {
        TraitId(self.traits.len() as u32)
    }

    /// Registers a trait definition in the registry.
    ///
    /// # Errors
    /// Returns `Err(existing_trait_id)` if a trait with the same name is already registered.
    pub fn register_trait(&mut self, trait_def: TraitDef) -> Result<TraitId, TraitId> {
        if let Some(&existing) = self.trait_names.get(&trait_def.name) {
            return Err(existing);
        }

        let id = trait_def.id;
        self.trait_names.insert(trait_def.name.clone(), id);
        self.traits.insert(id, trait_def);
        Ok(id)
    }

    /// Looks up a trait definition by name.
    pub fn lookup_trait(&self, name: &str) -> Option<&TraitDef> {
        let id = self.trait_names.get(name)?;
        self.traits.get(id)
    }

    /// Looks up a trait definition by [`TraitId`].
    pub fn lookup_trait_by_id(&self, id: TraitId) -> Option<&TraitDef> {
        self.traits.get(&id)
    }

    /// Registers a trait implementation block.
    ///
    /// # Errors
    /// Returns `Err(existing_impl)` if a trait implementation for the exact same target type
    /// is already registered (duplicate implementation detection).
    pub fn register_impl(&mut self, impl_def: TraitImplDef) -> Result<(), &TraitImplDef> {
        if let Some(existing) = self.implementations.iter().find(|existing| {
            existing.trait_id == impl_def.trait_id && existing.target_type == impl_def.target_type
        }) {
            return Err(existing);
        }

        self.implementations.push(impl_def);
        Ok(())
    }

    /// Registers an inherent implementation block (`impl Type`).
    pub fn register_inherent_impl(&mut self, inherent_def: InherentImplDef) {
        self.inherent_impls.push(inherent_def);
    }

    /// Returns `true` if the target semantic type implements the specified trait ID.
    pub fn implements_trait(&self, target_ty: TypeId, trait_id: TraitId) -> bool {
        self.implementations
            .iter()
            .any(|imp| imp.trait_id == trait_id && imp.target_type == target_ty)
    }

    /// Returns `true` if the target semantic type implements the specified trait name.
    pub fn implements_trait_named(&self, target_ty: TypeId, trait_name: &str) -> bool {
        if let Some(&id) = self.trait_names.get(trait_name) {
            self.implements_trait(target_ty, id)
        } else {
            false
        }
    }

    /// Returns an iterator over all registered traits.
    pub fn traits(&self) -> impl Iterator<Item = &TraitDef> {
        self.traits.values()
    }

    /// Returns a slice of all registered trait implementations.
    pub fn implementations(&self) -> &[TraitImplDef] {
        &self.implementations
    }

    /// Returns a slice of all registered inherent implementations.
    pub fn inherent_impls(&self) -> &[InherentImplDef] {
        &self.inherent_impls
    }

    /// Returns the number of registered traits.
    pub fn len(&self) -> usize {
        self.traits.len()
    }

    /// Returns `true` if no traits have been registered.
    pub fn is_empty(&self) -> bool {
        self.traits.is_empty()
    }
}
