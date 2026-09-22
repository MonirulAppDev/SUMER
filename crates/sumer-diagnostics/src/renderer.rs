//! Source-aware diagnostic rendering using [`SourceMap`].

use sumer_span::SourceMap;

use crate::collection::Diagnostics;
use crate::diagnostic::Diagnostic;
use crate::label::Label;
use crate::severity::Severity;

/// Terminal color output preference.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ColorMode {
    /// Automatically determine whether to use color (defaults to plain text).
    #[default]
    Auto,
    /// Always emit ANSI color escape sequences.
    Always,
    /// Never emit ANSI color escape sequences.
    Never,
}

/// Renderer that converts structured diagnostics into human-readable compiler output.
#[derive(Clone, Debug, Default)]
pub struct Renderer {
    color_mode: ColorMode,
}

impl Renderer {
    /// Creates a new `Renderer` with default color settings.
    pub fn new() -> Self {
        Self {
            color_mode: ColorMode::default(),
        }
    }

    /// Sets the color mode.
    pub fn with_color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    /// Returns `true` if ANSI color codes should be emitted.
    fn use_color(&self) -> bool {
        self.color_mode == ColorMode::Always
    }

    /// Renders a single diagnostic into a formatted string.
    pub fn render(&self, diagnostic: &Diagnostic, source_map: &SourceMap) -> String {
        let mut out = String::new();
        let use_color = self.use_color();

        // 1. Severity header (e.g. `error: message` or `error[E0001]: message`)
        self.render_header(&mut out, diagnostic, use_color);

        // 2. Collect all labels (primary first, then secondary)
        let mut labels: Vec<&Label> = Vec::new();
        if let Some(ref p) = diagnostic.primary {
            labels.push(p);
        }
        for s in &diagnostic.secondary {
            labels.push(s);
        }

        // 3. Render snippet if any labels with valid source files exist
        if !labels.is_empty() {
            self.render_labels_snippet(&mut out, &labels, source_map, use_color);
        }

        // 4. Render notes and help suggestions
        self.render_footer(&mut out, diagnostic, use_color);

        out
    }

    /// Renders all diagnostics in a collection into a formatted string.
    pub fn render_all(&self, diagnostics: &Diagnostics, source_map: &SourceMap) -> String {
        let mut out = String::new();
        for (i, diag) in diagnostics.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(&self.render(diag, source_map));
        }
        out
    }

    fn render_header(&self, out: &mut String, diagnostic: &Diagnostic, use_color: bool) {
        let sev_str = diagnostic.severity.as_str();
        let (prefix_color, reset) = if use_color {
            let color = match diagnostic.severity {
                Severity::Error => "\x1b[1;31m",   // bold red
                Severity::Warning => "\x1b[1;33m", // bold yellow
                Severity::Note => "\x1b[1;36m",    // bold cyan
                Severity::Help => "\x1b[1;32m",    // bold green
            };
            (color, "\x1b[0m")
        } else {
            ("", "")
        };

        if let Some(ref code) = diagnostic.code {
            out.push_str(&format!("{prefix_color}{sev_str}[{code}]{reset}: "));
        } else {
            out.push_str(&format!("{prefix_color}{sev_str}{reset}: "));
        }

        if use_color {
            out.push_str("\x1b[1m"); // bold message
            out.push_str(&diagnostic.message);
            out.push_str("\x1b[0m\n");
        } else {
            out.push_str(&diagnostic.message);
            out.push('\n');
        }
    }

    fn render_labels_snippet(
        &self,
        out: &mut String,
        labels: &[&Label],
        source_map: &SourceMap,
        use_color: bool,
    ) {
        // Find primary or first label to establish main location
        let first_label = labels[0];
        let Some(file) = source_map.get(first_label.span.source) else {
            return;
        };

        let primary_loc = file.offset_to_line_column(first_label.span.start as usize);
        let loc_str = if let Some(loc) = primary_loc {
            format!("{}:{}:{}", file.name(), loc.line, loc.column)
        } else {
            file.name().to_string()
        };

        // Determine which lines to display for this file
        let mut line_records: Vec<LineRenderRecord> = Vec::new();
        for label in labels {
            if label.span.source != file.id {
                continue;
            }
            let (start_line, start_col) = file
                .offset_to_line_column(label.span.start as usize)
                .map(|l| (l.line, l.column))
                .unwrap_or((1, 1));

            let end_offset = label.span.end.max(label.span.start);
            let (end_line, end_col) = file
                .offset_to_line_column(end_offset as usize)
                .map(|l| (l.line, l.column))
                .unwrap_or((start_line, start_col));

            line_records.push(LineRenderRecord {
                label,
                start_line,
                start_col,
                end_line,
                end_col,
            });
        }

        if line_records.is_empty() {
            return;
        }

        // Calculate max line number for gutter alignment
        let max_line = line_records.iter().map(|r| r.end_line).max().unwrap_or(1);
        let gutter_width = max_line.to_string().len().max(2);

        // Header: `  --> filename:line:col`
        let gutter_spaces = " ".repeat(gutter_width);
        out.push_str(&format!("{gutter_spaces}--> {loc_str}\n"));
        out.push_str(&format!("{gutter_spaces} |\n"));

        // Collect distinct lines to display, sorted
        let mut lines_to_show: Vec<usize> = Vec::new();
        for rec in &line_records {
            for line in rec.start_line..=rec.end_line {
                if !lines_to_show.contains(&line) {
                    lines_to_show.push(line);
                }
            }
        }
        lines_to_show.sort_unstable();

        let mut prev_line: Option<usize> = None;
        for &line_num in &lines_to_show {
            if let Some(prev) = prev_line
                && line_num > prev + 1
            {
                out.push_str("...\n");
            }
            prev_line = Some(line_num);

            let raw_text = file.line_text(line_num).unwrap_or("");
            let clean_line = raw_text.trim_end_matches(['\r', '\n']);

            // Print source line
            out.push_str(&format!("{line_num:>gutter_width$} | {clean_line}\n"));

            // Print underlines for labels on this line
            for rec in &line_records {
                if line_num >= rec.start_line && line_num <= rec.end_line {
                    let col_start = if line_num == rec.start_line {
                        rec.start_col
                    } else {
                        1
                    };

                    let line_char_count = clean_line.chars().count();
                    let col_end = if line_num == rec.end_line {
                        rec.end_col
                    } else {
                        line_char_count + 1
                    };

                    let width = if col_end > col_start {
                        col_end - col_start
                    } else {
                        1
                    };

                    let ch = if rec.label.is_primary { '^' } else { '-' };
                    let indent = " ".repeat(col_start.saturating_sub(1));
                    let underline_chars = ch.to_string().repeat(width);

                    let (color, reset) = if use_color {
                        if rec.label.is_primary {
                            ("\x1b[1;31m", "\x1b[0m")
                        } else {
                            ("\x1b[1;34m", "\x1b[0m")
                        }
                    } else {
                        ("", "")
                    };

                    if line_num == rec.end_line && !rec.label.message.is_empty() {
                        out.push_str(&format!(
                            "{gutter_spaces} | {indent}{color}{underline_chars}{reset} {}\n",
                            rec.label.message
                        ));
                    } else {
                        out.push_str(&format!(
                            "{gutter_spaces} | {indent}{color}{underline_chars}{reset}\n"
                        ));
                    }
                }
            }
        }
    }

    fn render_footer(&self, out: &mut String, diagnostic: &Diagnostic, use_color: bool) {
        if diagnostic.notes.is_empty() && diagnostic.helps.is_empty() {
            return;
        }

        let (note_color, help_color, reset) = if use_color {
            ("\x1b[1;36m", "\x1b[1;32m", "\x1b[0m")
        } else {
            ("", "", "")
        };

        for note in &diagnostic.notes {
            out.push_str(&format!("   = {note_color}note{reset}: {note}\n"));
        }

        for help in &diagnostic.helps {
            out.push_str(&format!("   = {help_color}help{reset}: {help}\n"));
        }
    }
}

struct LineRenderRecord<'a> {
    label: &'a Label,
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
}

/// Convenience function to render a diagnostic with default settings.
pub fn render(diagnostic: &Diagnostic, source_map: &SourceMap) -> String {
    Renderer::new().render(diagnostic, source_map)
}

/// Convenience function to render all diagnostics in a collection with default settings.
pub fn render_all(diagnostics: &Diagnostics, source_map: &SourceMap) -> String {
    Renderer::new().render_all(diagnostics, source_map)
}
