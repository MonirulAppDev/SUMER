//! Source identifier and span representations for SUMER.
//!
//! Spans represent byte ranges `[start, end)` in a registered source file.

use std::fmt;

/// Unique identifier for a source file registered in a [`SourceMap`](crate::SourceMap).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SourceId(pub u32);

impl SourceId {
    /// Creates a new `SourceId` with the given numeric identifier.
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the underlying raw `u32` value.
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SourceId({})", self.0)
    }
}

/// Represents a contiguous range of bytes in a source file.
///
/// Spans use byte offsets (not Unicode character indices):
/// - `start` is inclusive.
/// - `end` is exclusive.
///
/// For example, in `"hello"`, the span covering the whole word has `start = 0` and `end = 5`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Span {
    /// The source file this span originates from.
    pub source: SourceId,
    /// The starting byte offset (inclusive).
    pub start: u32,
    /// The ending byte offset (exclusive).
    pub end: u32,
}

impl Span {
    /// Creates a new `Span` with the given source ID and byte range.
    pub const fn new(source: SourceId, start: u32, end: u32) -> Self {
        Self { source, start, end }
    }

    /// Returns the source file ID this span belongs to.
    pub const fn source(&self) -> SourceId {
        self.source
    }

    /// Returns the starting byte offset (inclusive).
    pub const fn start(&self) -> u32 {
        self.start
    }

    /// Returns the ending byte offset (exclusive).
    pub const fn end(&self) -> u32 {
        self.end
    }

    /// Returns the byte length of this span.
    pub const fn len(&self) -> usize {
        self.end.saturating_sub(self.start) as usize
    }

    /// Returns `true` if the span has zero length (i.e., `start >= end`).
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Returns `true` if the span contains the given byte offset.
    ///
    /// The check is half-open: `position >= start && position < end`.
    pub const fn contains(&self, position: u32) -> bool {
        position >= self.start && position < self.end
    }

    /// Merges two spans from the same source file into a single span covering both.
    ///
    /// Returns `None` if `self.source != other.source`.
    pub fn join(&self, other: Span) -> Option<Span> {
        if self.source != other.source {
            return None;
        }

        let start = self.start.min(other.start);
        let end = self.end.max(other.end);
        Some(Span::new(self.source, start, end))
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}..{}", self.source, self.start, self.end)
    }
}

impl Default for Span {
    fn default() -> Self {
        Self {
            source: SourceId(0),
            start: 0,
            end: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_id_creation() {
        let id0 = SourceId::new(0);
        let id1 = SourceId(1);
        assert_eq!(id0.as_u32(), 0);
        assert_eq!(id1.as_u32(), 1);
        assert_ne!(id0, id1);
        assert!(id0 < id1);
    }

    #[test]
    fn test_span_creation_and_accessors() {
        let span = Span::new(SourceId(0), 10, 25);
        assert_eq!(span.source(), SourceId(0));
        assert_eq!(span.start(), 10);
        assert_eq!(span.end(), 25);
    }

    #[test]
    fn test_span_length() {
        let span = Span::new(SourceId(0), 5, 15);
        assert_eq!(span.len(), 10);
    }

    #[test]
    fn test_empty_span() {
        let empty = Span::new(SourceId(0), 10, 10);
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let inverted = Span::new(SourceId(0), 15, 10);
        assert!(inverted.is_empty());
        assert_eq!(inverted.len(), 0);

        let non_empty = Span::new(SourceId(0), 10, 11);
        assert!(!non_empty.is_empty());
        assert_eq!(non_empty.len(), 1);
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(SourceId(0), 10, 20);
        assert!(!span.contains(9));
        assert!(span.contains(10));
        assert!(span.contains(15));
        assert!(span.contains(19));
        assert!(!span.contains(20));
        assert!(!span.contains(21));
    }

    #[test]
    fn test_span_joining() {
        let s1 = Span::new(SourceId(0), 5, 10);
        let s2 = Span::new(SourceId(0), 15, 25);
        let joined = s1.join(s2).expect("should join same source spans");
        assert_eq!(joined.source(), SourceId(0));
        assert_eq!(joined.start(), 5);
        assert_eq!(joined.end(), 25);

        // Different sources must not join
        let s3 = Span::new(SourceId(1), 10, 30);
        assert_eq!(s1.join(s3), None);
    }
}
