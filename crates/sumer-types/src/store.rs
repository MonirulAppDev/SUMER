//! Type interning arena and formatting store.

use std::collections::HashMap;

use crate::generic::GenericParamId;
use crate::ty::Type;
use crate::type_id::TypeId;

/// Storage and interning arena for semantic [`Type`] instances.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeStore {
    types: Vec<Type>,
    interner: HashMap<Type, TypeId>,
    generic_params: Vec<String>,
}

impl Default for TypeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeStore {
    /// Creates a new `TypeStore` with all built-in primitive types pre-interned.
    pub fn new() -> Self {
        let mut store = Self {
            types: Vec::with_capacity(32),
            interner: HashMap::with_capacity(32),
            generic_params: Vec::new(),
        };

        // Populate standard primitive types matching TypeId constants 0..20
        store.pre_populate(Type::Unit, TypeId::UNIT);
        store.pre_populate(Type::Bool, TypeId::BOOL);
        store.pre_populate(Type::Int8, TypeId::INT8);
        store.pre_populate(Type::Int16, TypeId::INT16);
        store.pre_populate(Type::Int32, TypeId::INT32);
        store.pre_populate(Type::Int64, TypeId::INT64);
        store.pre_populate(Type::Int128, TypeId::INT128);
        store.pre_populate(Type::UInt8, TypeId::UINT8);
        store.pre_populate(Type::UInt16, TypeId::UINT16);
        store.pre_populate(Type::UInt32, TypeId::UINT32);
        store.pre_populate(Type::UInt64, TypeId::UINT64);
        store.pre_populate(Type::UInt128, TypeId::UINT128);
        store.pre_populate(Type::Float32, TypeId::FLOAT32);
        store.pre_populate(Type::Float64, TypeId::FLOAT64);
        store.pre_populate(Type::Char, TypeId::CHAR);
        store.pre_populate(Type::String, TypeId::STRING);
        store.pre_populate(Type::Byte, TypeId::BYTE);
        store.pre_populate(Type::Int, TypeId::INT);
        store.pre_populate(Type::UInt, TypeId::UINT);
        store.pre_populate(Type::Float, TypeId::FLOAT);
        store.pre_populate(Type::Error, TypeId::ERROR);

        store
    }

    fn pre_populate(&mut self, ty: Type, expected_id: TypeId) {
        let id = TypeId(self.types.len() as u32);
        assert_eq!(id, expected_id, "TypeId constant mismatch for {:?}", ty);
        self.types.push(ty.clone());
        self.interner.insert(ty, id);
    }

    /// Interns a semantic [`Type`], returning its unique [`TypeId`].
    pub fn intern(&mut self, ty: Type) -> TypeId {
        if let Some(&id) = self.interner.get(&ty) {
            return id;
        }

        let id = TypeId(self.types.len() as u32);
        self.types.push(ty.clone());
        self.interner.insert(ty, id);
        id
    }

    /// Retrieves the semantic [`Type`] corresponding to `id`.
    pub fn get(&self, id: TypeId) -> &Type {
        self.types
            .get(id.0 as usize)
            .expect("attempted to look up invalid TypeId")
    }

    /// Allocates and records a new generic parameter, returning its [`GenericParamId`].
    pub fn alloc_generic_param(&mut self, name: impl Into<String>) -> GenericParamId {
        let id = GenericParamId(self.generic_params.len() as u32);
        self.generic_params.push(name.into());
        id
    }

    /// Looks up the identifier name of a declared generic parameter.
    pub fn get_generic_param_name(&self, id: GenericParamId) -> Option<&str> {
        self.generic_params.get(id.0 as usize).map(|s| s.as_str())
    }

    /// Returns `true` if `expected` and `actual` are strictly equal, treating `Type::Error`
    /// as compatible for error recovery.
    pub fn type_equals(&self, a: TypeId, b: TypeId) -> bool {
        if a == b || a == TypeId::ERROR || b == TypeId::ERROR {
            return true;
        }

        match (self.get(a), self.get(b)) {
            (
                Type::Applied {
                    base: b1,
                    arguments: a1,
                },
                Type::Applied {
                    base: b2,
                    arguments: a2,
                },
            ) => {
                self.type_equals(*b1, *b2)
                    && a1.len() == a2.len()
                    && a1.iter().zip(a2).all(|(&x, &y)| self.type_equals(x, y))
            }
            (Type::Applied { base, arguments }, Type::Generic { name, args }) => {
                if let Type::Named(base_name) = self.get(*base) {
                    base_name == name
                        && arguments.len() == args.len()
                        && arguments
                            .iter()
                            .zip(args)
                            .all(|(&x, &y)| self.type_equals(x, y))
                } else {
                    false
                }
            }
            (Type::Generic { name, args }, Type::Applied { base, arguments }) => {
                if let Type::Named(base_name) = self.get(*base) {
                    base_name == name
                        && arguments.len() == args.len()
                        && arguments
                            .iter()
                            .zip(args)
                            .all(|(&x, &y)| self.type_equals(x, y))
                } else {
                    false
                }
            }
            (Type::Optional(i1), Type::Optional(i2)) => self.type_equals(*i1, *i2),
            (Type::List(e1), Type::List(e2)) => self.type_equals(*e1, *e2),
            (Type::Array { element: e1 }, Type::Array { element: e2 }) => {
                self.type_equals(*e1, *e2)
            }
            (
                Type::Map {
                    key: k1,
                    value: v1,
                },
                Type::Map {
                    key: k2,
                    value: v2,
                },
            ) => self.type_equals(*k1, *k2) && self.type_equals(*v1, *v2),
            (
                Type::Reference {
                    mutable: m1,
                    inner: i1,
                },
                Type::Reference {
                    mutable: m2,
                    inner: i2,
                },
            ) => m1 == m2 && self.type_equals(*i1, *i2),
            (
                Type::Function {
                    params: p1,
                    return_type: r1,
                },
                Type::Function {
                    params: p2,
                    return_type: r2,
                },
            ) => {
                p1.len() == p2.len()
                    && p1.iter().zip(p2).all(|(&x, &y)| self.type_equals(x, y))
                    && self.type_equals(*r1, *r2)
            }
            (Type::Tuple(t1), Type::Tuple(t2)) => {
                t1.len() == t2.len()
                    && t1.iter().zip(t2).all(|(&x, &y)| self.type_equals(x, y))
            }
            (Type::GenericParam(p1), Type::GenericParam(p2)) => p1 == p2,
            _ => false,
        }
    }

    /// Checks whether an expression of type `actual` is assignable to an expected type `expected`.
    pub fn is_assignable(&self, expected: TypeId, actual: TypeId) -> bool {
        if self.type_equals(expected, actual) {
            return true;
        }

        // Allow Option<T> to accept Option<T> (covered by equality)
        false
    }

    /// Formats a [`TypeId`] into a concise, human-readable compiler type representation.
    pub fn format_type(&self, id: TypeId) -> String {
        match self.get(id) {
            Type::Unit => "Unit".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::Int8 => "Int8".to_string(),
            Type::Int16 => "Int16".to_string(),
            Type::Int32 => "Int32".to_string(),
            Type::Int64 => "Int64".to_string(),
            Type::Int128 => "Int128".to_string(),
            Type::UInt8 => "UInt8".to_string(),
            Type::UInt16 => "UInt16".to_string(),
            Type::UInt32 => "UInt32".to_string(),
            Type::UInt64 => "UInt64".to_string(),
            Type::UInt128 => "UInt128".to_string(),
            Type::Float32 => "Float32".to_string(),
            Type::Float64 => "Float64".to_string(),
            Type::Char => "Char".to_string(),
            Type::String => "String".to_string(),
            Type::Byte => "Byte".to_string(),
            Type::Int => "Int".to_string(),
            Type::UInt => "UInt".to_string(),
            Type::Float => "Float".to_string(),
            Type::Named(name) => name.clone(),
            Type::GenericParam(param_id) => {
                if let Some(name) = self.get_generic_param_name(*param_id) {
                    name.to_string()
                } else {
                    format!("T{}", param_id.0)
                }
            }
            Type::Applied { base, arguments } => {
                let base_fmt = self.format_type(*base);
                let formatted_args: Vec<String> =
                    arguments.iter().map(|&arg| self.format_type(arg)).collect();
                format!("{}<{}>", base_fmt, formatted_args.join(", "))
            }
            Type::Generic { name, args } => {
                let formatted_args: Vec<String> =
                    args.iter().map(|&arg| self.format_type(arg)).collect();
                format!("{}<{}>", name, formatted_args.join(", "))
            }
            Type::Reference { mutable, inner } => {
                let inner_fmt = self.format_type(*inner);
                if *mutable {
                    format!("&mut {}", inner_fmt)
                } else {
                    format!("&{}", inner_fmt)
                }
            }
            Type::Optional(inner) => {
                format!("Option<{}>", self.format_type(*inner))
            }
            Type::Function {
                params,
                return_type,
            } => {
                let param_strs: Vec<String> = params.iter().map(|&p| self.format_type(p)).collect();
                format!(
                    "fn({}) -> {}",
                    param_strs.join(", "),
                    self.format_type(*return_type)
                )
            }
            Type::Tuple(elements) => {
                let elem_strs: Vec<String> =
                    elements.iter().map(|&e| self.format_type(e)).collect();
                format!("({})", elem_strs.join(", "))
            }
            Type::Array { element } => {
                format!("[{}]", self.format_type(*element))
            }
            Type::List(element) => {
                format!("List<{}>", self.format_type(*element))
            }
            Type::Map { key, value } => {
                format!(
                    "Map<{}, {}>",
                    self.format_type(*key),
                    self.format_type(*value)
                )
            }
            Type::Set(element) => {
                format!("Set<{}>", self.format_type(*element))
            }
            Type::Error => "{error}".to_string(),
        }
    }
}
