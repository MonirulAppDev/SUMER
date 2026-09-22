//! Diagnostic collection and aggregation.

use std::slice::Iter;

use crate::diagnostic::Diagnostic;

/// A collection of compiler diagnostics emitted during compilation phases.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    /// Creates an empty collection of diagnostics.
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// Adds a diagnostic to the collection.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Returns `true` if any diagnostic in the collection is an error.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }

    /// Returns `true` if any diagnostic in the collection is a warning.
    pub fn has_warnings(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_warning())
    }

    /// Returns `true` if the collection contains no diagnostics.
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Returns the number of diagnostics in the collection.
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Returns a slice of the stored diagnostics.
    pub fn as_slice(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns an iterator over the stored diagnostics.
    pub fn iter(&self) -> Iter<'_, Diagnostic> {
        self.diagnostics.iter()
    }

    /// Clears all diagnostics from the collection.
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }
}

impl<'a> IntoIterator for &'a Diagnostics {
    type Item = &'a Diagnostic;
    type IntoIter = Iter<'a, Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

impl Extend<Diagnostic> for Diagnostics {
    fn extend<T: IntoIterator<Item = Diagnostic>>(&mut self, iter: T) {
        self.diagnostics.extend(iter);
    }
}
