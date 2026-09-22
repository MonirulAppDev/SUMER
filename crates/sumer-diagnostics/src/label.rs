//! Source code annotations and labels within diagnostics.

use sumer_span::Span;

/// A labeled source span highlighting an issue or related context in a source file.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Label {
    /// The source span being highlighted.
    pub span: Span,
    /// Message displayed next to or under the highlighted span.
    pub message: String,
    /// Whether this is the primary point of failure or an explanatory secondary label.
    pub is_primary: bool,
}

impl Label {
    /// Creates a new label.
    pub fn new(span: Span, message: impl Into<String>, is_primary: bool) -> Self {
        Self {
            span,
            message: message.into(),
            is_primary,
        }
    }

    /// Creates a primary label (typically underlined with `^`).
    pub fn primary(span: Span, message: impl Into<String>) -> Self {
        Self::new(span, message, true)
    }

    /// Creates a secondary / contextual label (typically underlined with `-`).
    pub fn secondary(span: Span, message: impl Into<String>) -> Self {
        Self::new(span, message, false)
    }
}
