//! Semantic type representations decoupled from syntax AST nodes.

use crate::generic::GenericParamId;
use crate::type_id::TypeId;

/// The semantic representation of a type in the SUMER compiler.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    /// Unit type representing empty/void value `()`.
    Unit,
    /// Boolean type `Bool`.
    Bool,

    /// 8-bit signed integer `Int8`.
    Int8,
    /// 16-bit signed integer `Int16`.
    Int16,
    /// 32-bit signed integer `Int32`.
    Int32,
    /// 64-bit signed integer `Int64`.
    Int64,
    /// 128-bit signed integer `Int128`.
    Int128,

    /// 8-bit unsigned integer `UInt8`.
    UInt8,
    /// 16-bit unsigned integer `UInt16`.
    UInt16,
    /// 32-bit unsigned integer `UInt32`.
    UInt32,
    /// 64-bit unsigned integer `UInt64`.
    UInt64,
    /// 128-bit unsigned integer `UInt128`.
    UInt128,

    /// 32-bit floating point `Float32`.
    Float32,
    /// 64-bit floating point `Float64`.
    Float64,

    /// Unicode character `Char`.
    Char,
    /// UTF-8 string `String`.
    String,
    /// Byte type `Byte`.
    Byte,

    /// Default signed integer `Int`.
    Int,
    /// Default unsigned integer `UInt`.
    UInt,
    /// Default floating point `Float`.
    Float,

    /// User-defined named type (e.g. struct, enum, trait).
    Named(std::string::String),

    /// Generic type parameter identity (e.g. `T`).
    GenericParam(GenericParamId),

    /// Concrete applied generic type (e.g. `Box<Int>`, `Result<User, Error>`).
    Applied {
        base: TypeId,
        arguments: Vec<TypeId>,
    },

    /// Legacy / named generic type helper (e.g. `Result<T, E>`).
    Generic {
        name: std::string::String,
        args: Vec<TypeId>,
    },

    /// Reference type (`&T` or `&mut T`).
    Reference { mutable: bool, inner: TypeId },

    /// Optional type (`Option<T>` or `T?`).
    Optional(TypeId),

    /// Function signature type (`(T1, T2) -> Ret`).
    Function {
        params: Vec<TypeId>,
        return_type: TypeId,
    },

    /// Tuple product type (`(T1, T2)`).
    Tuple(Vec<TypeId>),

    /// Fixed array type (`[T]`).
    Array { element: TypeId },

    /// Dynamic list collection type (`List<T>`).
    List(TypeId),

    /// Map collection type (`Map<Key, Value>`).
    Map { key: TypeId, value: TypeId },

    /// Set collection type (`Set<T>`).
    Set(TypeId),

    /// Error sentinel type for resilient error recovery.
    Error,
}

impl Type {
    /// Returns `true` if this type represents any signed or unsigned integer.
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Self::Int8
                | Self::Int16
                | Self::Int32
                | Self::Int64
                | Self::Int128
                | Self::UInt8
                | Self::UInt16
                | Self::UInt32
                | Self::UInt64
                | Self::UInt128
                | Self::Byte
                | Self::Int
                | Self::UInt
        )
    }

    /// Returns `true` if this type represents any floating-point number.
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float32 | Self::Float64 | Self::Float)
    }

    /// Returns `true` if this type represents any numeric type (integer or float).
    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    /// Returns `true` if this type is `Bool`.
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool)
    }

    /// Returns `true` if this type is `String`.
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String)
    }

    /// Returns `true` if this type is `Unit`.
    pub fn is_unit(&self) -> bool {
        matches!(self, Self::Unit)
    }

    /// Returns `true` if this type is the error sentinel.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }
}
