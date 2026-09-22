//! Diagnostic representation and builder API.

use sumer_span::Span;

use crate::label::Label;
use crate::severity::Severity;

/// A structured compiler diagnostic with primary message, labels, notes, and suggestions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// Diagnostic severity level (`Error`, `Warning`, `Note`, `Help`).
    pub severity: Severity,
    /// Optional error/warning code (e.g. `"E0001"`).
    pub code: Option<String>,
    /// Primary high-level diagnostic message.
    pub message: String,
    /// Primary location and label explaining the issue.
    pub primary: Option<Label>,
    /// Additional context locations with explanatory labels.
    pub secondary: Vec<Label>,
    /// Informational notes attached to this diagnostic.
    pub notes: Vec<String>,
    /// Actionable help suggestions.
    pub helps: Vec<String>,
}

impl Diagnostic {
    /// Creates a new diagnostic with the given severity and message.
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        Self {
            severity,
            code: None,
            message: message.into(),
            primary: None,
            secondary: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
        }
    }

    /// Creates a new error diagnostic.
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(Severity::Error, message)
    }

    /// Creates a new warning diagnostic.
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, message)
    }

    /// Creates a standalone informational note diagnostic.
    pub fn note(message: impl Into<String>) -> Self {
        Self::new(Severity::Note, message)
    }

    /// Creates a standalone help diagnostic.
    pub fn help(message: impl Into<String>) -> Self {
        Self::new(Severity::Help, message)
    }

    /// Attaches an error code (e.g. `"E0001"`).
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Sets the primary source span and label message.
    pub fn with_primary(mut self, span: Span, message: impl Into<String>) -> Self {
        self.primary = Some(Label::primary(span, message));
        self
    }

    /// Adds a secondary source span and contextual message.
    pub fn with_secondary(mut self, span: Span, message: impl Into<String>) -> Self {
        self.secondary.push(Label::secondary(span, message));
        self
    }

    /// Adds an arbitrary label (primary or secondary).
    pub fn with_label(mut self, label: Label) -> Self {
        if label.is_primary {
            self.primary = Some(label);
        } else {
            self.secondary.push(label);
        }
        self
    }

    /// Adds an informational note.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Adds multiple informational notes.
    pub fn with_notes<I, S>(mut self, notes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for note in notes {
            self.notes.push(note.into());
        }
        self
    }

    /// Adds an actionable suggestion / help message.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.helps.push(help.into());
        self
    }

    /// Adds multiple help messages.
    pub fn with_helps<I, S>(mut self, helps: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for help in helps {
            self.helps.push(help.into());
        }
        self
    }

    /// Returns `true` if this diagnostic is an error.
    pub fn is_error(&self) -> bool {
        self.severity.is_error()
    }

    /// Returns `true` if this diagnostic is a warning.
    pub fn is_warning(&self) -> bool {
        self.severity.is_warning()
    }
}
