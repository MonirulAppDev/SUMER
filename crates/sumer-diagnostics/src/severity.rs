//! Diagnostic severity levels.

use std::fmt;

/// Severity classification of a compiler diagnostic.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// An error preventing compilation or execution.
    Error,
    /// A potential issue that does not prevent compilation.
    Warning,
    /// Informational note associated with a diagnostic or standalone.
    Note,
    /// A suggestion or hint to help resolve an issue.
    Help,
}

impl Severity {
    /// Returns the standard lowercase string representation (e.g. `"error"`).
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Note => "note",
            Self::Help => "help",
        }
    }

    /// Returns `true` if this severity is an error.
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }

    /// Returns `true` if this severity is a warning.
    pub const fn is_warning(&self) -> bool {
        matches!(self, Self::Warning)
    }

    /// Returns `true` if this severity is an informational note.
    pub const fn is_note(&self) -> bool {
        matches!(self, Self::Note)
    }

    /// Returns `true` if this severity is a help message.
    pub const fn is_help(&self) -> bool {
        matches!(self, Self::Help)
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
