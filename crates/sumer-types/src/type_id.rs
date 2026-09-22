//! Compact TypeId identifier representing interned semantic types.

use std::fmt;

/// Compact 32-bit identifier for an interned semantic type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(pub u32);

impl TypeId {
    /// Unit type representing empty/void value `()`.
    pub const UNIT: Self = Self(0);
    /// Boolean type `Bool`.
    pub const BOOL: Self = Self(1);
    /// 8-bit signed integer `Int8`.
    pub const INT8: Self = Self(2);
    /// 16-bit signed integer `Int16`.
    pub const INT16: Self = Self(3);
    /// 32-bit signed integer `Int32`.
    pub const INT32: Self = Self(4);
    /// 64-bit signed integer `Int64`.
    pub const INT64: Self = Self(5);
    /// 128-bit signed integer `Int128`.
    pub const INT128: Self = Self(6);
    /// 8-bit unsigned integer `UInt8`.
    pub const UINT8: Self = Self(7);
    /// 16-bit unsigned integer `UInt16`.
    pub const UINT16: Self = Self(8);
    /// 32-bit unsigned integer `UInt32`.
    pub const UINT32: Self = Self(9);
    /// 64-bit unsigned integer `UInt64`.
    pub const UINT64: Self = Self(10);
    /// 128-bit unsigned integer `UInt128`.
    pub const UINT128: Self = Self(11);
    /// 32-bit floating point `Float32`.
    pub const FLOAT32: Self = Self(12);
    /// 64-bit floating point `Float64`.
    pub const FLOAT64: Self = Self(13);
    /// Character type `Char`.
    pub const CHAR: Self = Self(14);
    /// UTF-8 string type `String`.
    pub const STRING: Self = Self(15);
    /// Byte type `Byte` (alias for 8-bit unsigned).
    pub const BYTE: Self = Self(16);
    /// Default signed integer `Int`.
    pub const INT: Self = Self(17);
    /// Default unsigned integer `UInt`.
    pub const UINT: Self = Self(18);
    /// Default floating point `Float`.
    pub const FLOAT: Self = Self(19);
    /// Error sentinel type used for recovery during malformed code analysis.
    pub const ERROR: Self = Self(20);
}

impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TypeId({})", self.0)
    }
}
