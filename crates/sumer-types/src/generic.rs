//! Generic parameter identities and recursive type substitution engine.

use std::collections::HashMap;
use std::fmt;

use crate::store::TypeStore;
use crate::ty::Type;
use crate::type_id::TypeId;

/// Maximum recursion depth allowed during type substitution to prevent infinite cycles.
const MAX_SUBSTITUTION_DEPTH: usize = 64;

/// Compact stable 32-bit identifier for a generic type parameter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GenericParamId(pub u32);

impl fmt::Display for GenericParamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Param({})", self.0)
    }
}

/// A mapping from generic type parameters to concrete semantic types.
///
/// Provides recursive type substitution across references, optionals,
/// functions, tuples, collections, and nested applied generic types.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeSubstitution {
    mappings: HashMap<GenericParamId, TypeId>,
}

impl TypeSubstitution {
    /// Creates an empty `TypeSubstitution`.
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Creates a substitution from an existing parameter-to-type mapping.
    pub fn from_mapping(mappings: HashMap<GenericParamId, TypeId>) -> Self {
        Self { mappings }
    }

    /// Associates a generic parameter with a concrete type.
    pub fn insert(&mut self, param: GenericParamId, concrete_type: TypeId) {
        self.mappings.insert(param, concrete_type);
    }

    /// Retrieves the concrete type mapped to the given generic parameter, if any.
    pub fn get(&self, param: GenericParamId) -> Option<TypeId> {
        self.mappings.get(&param).copied()
    }

    /// Returns the number of parameter mappings in this substitution.
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// Returns `true` if this substitution contains no mappings.
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Returns an iterator over all parameter mappings in this substitution.
    pub fn iter(&self) -> impl Iterator<Item = (&GenericParamId, &TypeId)> {
        self.mappings.iter()
    }

    /// Recursively applies this substitution to the given [`TypeId`], returning the substituted type.
    ///
    /// Protects against infinite recursion cycles using a traversal depth limit.
    pub fn apply(&self, ty: TypeId, store: &mut TypeStore) -> TypeId {
        if self.is_empty() {
            return ty;
        }
        self.apply_rec(ty, store, 0)
    }

    fn apply_rec(&self, ty: TypeId, store: &mut TypeStore, depth: usize) -> TypeId {
        if depth > MAX_SUBSTITUTION_DEPTH {
            return TypeId::ERROR;
        }

        match store.get(ty).clone() {
            Type::GenericParam(param_id) => {
                if let Some(&concrete) = self.mappings.get(&param_id) {
                    if concrete != ty {
                        self.apply_rec(concrete, store, depth + 1)
                    } else {
                        concrete
                    }
                } else {
                    ty
                }
            }
            Type::Applied { base, arguments } => {
                let new_base = self.apply_rec(base, store, depth + 1);
                let mut changed = new_base != base;
                let mut new_args = Vec::with_capacity(arguments.len());
                for &arg in &arguments {
                    let new_arg = self.apply_rec(arg, store, depth + 1);
                    if new_arg != arg {
                        changed = true;
                    }
                    new_args.push(new_arg);
                }
                if changed {
                    store.intern(Type::Applied {
                        base: new_base,
                        arguments: new_args,
                    })
                } else {
                    ty
                }
            }
            Type::Reference { mutable, inner } => {
                let new_inner = self.apply_rec(inner, store, depth + 1);
                if new_inner != inner {
                    store.intern(Type::Reference {
                        mutable,
                        inner: new_inner,
                    })
                } else {
                    ty
                }
            }
            Type::Optional(inner) => {
                let new_inner = self.apply_rec(inner, store, depth + 1);
                if new_inner != inner {
                    store.intern(Type::Optional(new_inner))
                } else {
                    ty
                }
            }
            Type::Function {
                params,
                return_type,
            } => {
                let new_ret = self.apply_rec(return_type, store, depth + 1);
                let mut changed = new_ret != return_type;
                let mut new_params = Vec::with_capacity(params.len());
                for &p in &params {
                    let new_p = self.apply_rec(p, store, depth + 1);
                    if new_p != p {
                        changed = true;
                    }
                    new_params.push(new_p);
                }
                if changed {
                    store.intern(Type::Function {
                        params: new_params,
                        return_type: new_ret,
                    })
                } else {
                    ty
                }
            }
            Type::Tuple(elements) => {
                let mut changed = false;
                let mut new_elems = Vec::with_capacity(elements.len());
                for &e in &elements {
                    let new_e = self.apply_rec(e, store, depth + 1);
                    if new_e != e {
                        changed = true;
                    }
                    new_elems.push(new_e);
                }
                if changed {
                    store.intern(Type::Tuple(new_elems))
                } else {
                    ty
                }
            }
            Type::Array { element } => {
                let new_elem = self.apply_rec(element, store, depth + 1);
                if new_elem != element {
                    store.intern(Type::Array { element: new_elem })
                } else {
                    ty
                }
            }
            Type::List(element) => {
                let new_elem = self.apply_rec(element, store, depth + 1);
                if new_elem != element {
                    store.intern(Type::List(new_elem))
                } else {
                    ty
                }
            }
            Type::Map { key, value } => {
                let new_key = self.apply_rec(key, store, depth + 1);
                let new_val = self.apply_rec(value, store, depth + 1);
                if new_key != key || new_val != value {
                    store.intern(Type::Map {
                        key: new_key,
                        value: new_val,
                    })
                } else {
                    ty
                }
            }
            Type::Set(element) => {
                let new_elem = self.apply_rec(element, store, depth + 1);
                if new_elem != element {
                    store.intern(Type::Set(new_elem))
                } else {
                    ty
                }
            }
            _ => ty,
        }
    }
}
