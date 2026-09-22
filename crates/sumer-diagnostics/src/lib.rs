//! Foundational compiler diagnostics and source-aware error reporting for SUMER.
//!
//! Provides the diagnostic data model ([`Diagnostic`], [`Severity`], [`Label`], [`Diagnostics`])
//! and source-aware formatting engine ([`Renderer`], [`render`], [`render_all`]) using [`SourceMap`].

pub mod collection;
pub mod diagnostic;
pub mod label;
pub mod renderer;
pub mod severity;

pub use collection::Diagnostics;
pub use diagnostic::Diagnostic;
pub use label::Label;
pub use renderer::{ColorMode, Renderer, render, render_all};
pub use severity::Severity;

#[cfg(test)]
mod tests {
    use super::*;
    use sumer_span::{SourceMap, Span};

    #[test]
    fn test_severity_levels() {
        assert!(Severity::Error.is_error());
        assert!(!Severity::Error.is_warning());
        assert_eq!(Severity::Error.as_str(), "error");

        assert!(Severity::Warning.is_warning());
        assert_eq!(Severity::Warning.as_str(), "warning");

        assert!(Severity::Note.is_note());
        assert_eq!(Severity::Note.as_str(), "note");

        assert!(Severity::Help.is_help());
        assert_eq!(Severity::Help.as_str(), "help");
    }

    #[test]
    fn test_diagnostic_builder_api() {
        let mut source_map = SourceMap::new();
        let id = source_map.add("test.sm", "let x = ;");
        let span = Span::new(id, 8, 9);
        let sec_span = Span::new(id, 0, 3);

        let diag = Diagnostic::error("unexpected token")
            .with_code("E0001")
            .with_primary(span, "expected expression")
            .with_secondary(sec_span, "variable declared here")
            .with_note("an expression must follow '=' in a variable declaration")
            .with_help("provide a value such as '10'");

        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.code.as_deref(), Some("E0001"));
        assert_eq!(diag.message, "unexpected token");
        assert!(diag.primary.is_some());
        assert_eq!(diag.secondary.len(), 1);
        assert_eq!(diag.notes.len(), 1);
        assert_eq!(diag.helps.len(), 1);

        let rendered = render(&diag, &source_map);
        assert!(rendered.contains("error[E0001]: unexpected token"));
        assert!(rendered.contains("--> test.sm:1:9"));
        assert!(rendered.contains("let x = ;"));
        assert!(rendered.contains("^ expected expression"));
        assert!(rendered.contains("--- variable declared here"));
        assert!(
            rendered.contains("= note: an expression must follow '=' in a variable declaration")
        );
        assert!(rendered.contains("= help: provide a value such as '10'"));
    }

    #[test]
    fn test_diagnostics_collection() {
        let mut diags = Diagnostics::new();
        assert!(diags.is_empty());
        assert_eq!(diags.len(), 0);
        assert!(!diags.has_errors());
        assert!(!diags.has_warnings());

        diags.push(Diagnostic::warning("unused variable"));
        assert_eq!(diags.len(), 1);
        assert!(diags.has_warnings());
        assert!(!diags.has_errors());

        diags.push(Diagnostic::error("type mismatch"));
        assert_eq!(diags.len(), 2);
        assert!(diags.has_errors());

        let count = diags.iter().count();
        assert_eq!(count, 2);

        diags.clear();
        assert!(diags.is_empty());
    }

    #[test]
    fn test_render_single_line_diagnostic() {
        let mut source_map = SourceMap::new();
        let src = "fn main() {\n    let x = ;\n}\n";
        let id = source_map.add("hello.sm", src);
        let span = Span::new(id, 24, 25); // `;` on line 2

        let diag = Diagnostic::error("unexpected token").with_primary(span, "expected expression");

        let rendered = render(&diag, &source_map);
        let expected = "\
error: unexpected token
  --> hello.sm:2:13
   |
 2 |     let x = ;
   |             ^ expected expression
";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_render_zero_length_span() {
        let mut source_map = SourceMap::new();
        let src = "fn main() {\n";
        let id = source_map.add("test.sm", src);
        let span = Span::new(id, 12, 12); // EOF position

        let diag = Diagnostic::error("expected '}'").with_primary(span, "expected '}' here");

        let rendered = render(&diag, &source_map);
        assert!(rendered.contains("error: expected '}'"));
        assert!(rendered.contains("--> test.sm:2:1"));
        assert!(rendered.contains("^ expected '}' here"));
    }

    #[test]
    fn test_render_multi_label_with_gap() {
        let mut source_map = SourceMap::new();
        let src = "\
let name = \"B\"
let other = 1
let more = 2
let name = \"A\"
";
        let id = source_map.add("example.sm", src);
        let first_decl = Span::new(id, 4, 8); // `name` on line 1
        let second_decl = Span::new(id, 46, 50); // `name` on line 4

        let diag = Diagnostic::error("duplicate declaration")
            .with_secondary(first_decl, "previous declaration is here")
            .with_primary(second_decl, "already declared here");

        let rendered = render(&diag, &source_map);
        assert!(rendered.contains("error: duplicate declaration"));
        assert!(rendered.contains("--> example.sm:4:5"));
        assert!(rendered.contains("1 | let name = \"B\""));
        assert!(rendered.contains("---- previous declaration is here"));
        assert!(rendered.contains("..."));
        assert!(rendered.contains("4 | let name = \"A\""));
        assert!(rendered.contains("^^^^ already declared here"));
    }

    #[test]
    fn test_render_utf8_and_crlf() {
        let mut source_map = SourceMap::new();
        let src = "let привет = 10\r\nlet 🦀 = ;\r\n";
        let id = source_map.add("unicode.sm", src);
        let span = Span::new(id, 30, 31); // `;` after crab emoji

        let diag = Diagnostic::error("expected expression").with_primary(span, "value needed");

        let rendered = render(&diag, &source_map);
        assert!(rendered.contains("error: expected expression"));
        assert!(rendered.contains("2 | let 🦀 = ;"));
        assert!(rendered.contains("^ value needed"));
        assert!(!rendered.contains('\r')); // CRLF carriage return stripped from display
    }

    #[test]
    fn test_render_all_multiple_diagnostics() {
        let mut source_map = SourceMap::new();
        let id = source_map.add("multi.sm", "let a = 1\nlet b = 2\n");

        let mut diags = Diagnostics::new();
        diags.push(Diagnostic::error("first error").with_primary(Span::new(id, 0, 3), "first"));
        diags.push(
            Diagnostic::warning("second warning").with_primary(Span::new(id, 10, 13), "second"),
        );

        let rendered = render_all(&diags, &source_map);
        assert!(rendered.contains("error: first error"));
        assert!(rendered.contains("warning: second warning"));
    }
}
