//! Source file and source map management for the SUMER compiler.
//!
//! Provides [`SourceFile`], [`SourceMap`], and [`LineColumn`] location conversion.

use std::fmt;

use crate::span::{SourceId, Span};

/// Human-readable 1-based line and column coordinates within a source file.
///
/// Both `line` and `column` are 1-based for consistent compiler diagnostics
/// (e.g. `file.sm:10:15`).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LineColumn {
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number (counted in Unicode characters/codepoints).
    pub column: usize,
}

impl LineColumn {
    /// Creates a new `LineColumn` coordinates instance.
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl fmt::Display for LineColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// A single source file tracked by the compiler.
///
/// Holds the source identifier, filename/path, full raw text, and precomputed
/// line-start offsets to accelerate line/column queries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile {
    /// Unique identifier for this source file.
    pub id: SourceId,
    /// Human-readable name or path of the source file (e.g. `"hello.sm"`).
    pub name: String,
    /// Complete UTF-8 source text.
    pub text: String,
    /// Byte offsets where each line begins (always starts with `0`).
    line_starts: Vec<u32>,
}

impl SourceFile {
    /// Creates a new `SourceFile` and indexes its line-start positions.
    pub fn new(id: SourceId, name: impl Into<String>, text: impl Into<String>) -> Self {
        let name = name.into();
        let text = text.into();
        let line_starts = Self::compute_line_starts(&text);

        Self {
            id,
            name,
            text,
            line_starts,
        }
    }

    /// Precomputes the byte offset of each line start in the source text.
    ///
    /// The first line always starts at byte offset 0.
    /// Newlines (`\n`) determine line breaks. For Windows `\r\n` files,
    /// the line starts immediately after the `\n`, ensuring CRLF is treated
    /// as a single line boundary without modifying source text.
    fn compute_line_starts(text: &str) -> Vec<u32> {
        let mut starts = vec![0];
        for (i, byte) in text.as_bytes().iter().enumerate() {
            if *byte == b'\n' {
                starts.push((i + 1) as u32);
            }
        }
        starts
    }

    /// Returns the source identifier.
    pub const fn id(&self) -> SourceId {
        self.id
    }

    /// Returns the source file name or path.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a slice of the entire source text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the total length of the source text in bytes.
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Returns `true` if the source text is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Returns the total number of lines in this file (at least 1).
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Converts a byte offset into 1-based [`LineColumn`] coordinates.
    ///
    /// Returns `None` if `offset > self.text.len()`.
    ///
    /// - **Line**: 1-based index determined via binary search over precomputed line starts.
    /// - **Column**: 1-based index determined by counting Unicode characters (`char`)
    ///   from the line start up to `offset`.
    pub fn offset_to_line_column(&self, offset: usize) -> Option<LineColumn> {
        if offset > self.text.len() {
            return None;
        }

        let offset_u32 = offset as u32;
        let line_idx = match self.line_starts.binary_search(&offset_u32) {
            Ok(exact) => exact,
            Err(next) => next - 1,
        };

        let line = line_idx + 1;
        let line_start = self.line_starts[line_idx] as usize;

        // Count Unicode characters from line_start to offset safely without panicking.
        let mut column = 1;
        for (byte_offset, _) in self.text[line_start..].char_indices() {
            let current_byte = line_start + byte_offset;
            if current_byte >= offset {
                break;
            }
            column += 1;
        }

        Some(LineColumn::new(line, column))
    }

    /// Returns the raw text of the specified 1-based line number.
    ///
    /// Returns `None` if `line == 0` or `line > line_count()`.
    pub fn line_text(&self, line: usize) -> Option<&str> {
        if line == 0 || line > self.line_starts.len() {
            return None;
        }

        let start = self.line_starts[line - 1] as usize;
        let end = if line < self.line_starts.len() {
            self.line_starts[line] as usize
        } else {
            self.text.len()
        };

        Some(&self.text[start..end])
    }
}

/// Stores registered source files and maps sequential [`SourceId`]s to them.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    /// Creates an empty `SourceMap`.
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Registers a new source file and assigns it a deterministic, sequential [`SourceId`].
    ///
    /// The first registered file gets `SourceId(0)`, the second `SourceId(1)`, etc.
    pub fn add(&mut self, name: impl Into<String>, text: impl Into<String>) -> SourceId {
        let id = SourceId(self.files.len() as u32);
        let file = SourceFile::new(id, name, text);
        self.files.push(file);
        id
    }

    /// Retrieves an immutable reference to a source file by its [`SourceId`].
    pub fn get(&self, id: SourceId) -> Option<&SourceFile> {
        self.files.get(id.0 as usize)
    }

    /// Retrieves a mutable reference to a source file by its [`SourceId`].
    pub fn get_mut(&mut self, id: SourceId) -> Option<&mut SourceFile> {
        self.files.get_mut(id.0 as usize)
    }

    /// Retrieves the raw source text for the specified [`SourceId`].
    pub fn get_text(&self, id: SourceId) -> Option<&str> {
        self.get(id).map(|f| f.text())
    }

    /// Retrieves the source file name for the specified [`SourceId`].
    pub fn get_name(&self, id: SourceId) -> Option<&str> {
        self.get(id).map(|f| f.name())
    }

    /// Returns the number of registered source files.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Returns `true` if no source files are registered.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Converts a byte offset within a source file to 1-based [`LineColumn`].
    pub fn offset_to_line_column(&self, id: SourceId, offset: usize) -> Option<LineColumn> {
        self.get(id)
            .and_then(|file| file.offset_to_line_column(offset))
    }

    /// Converts the start offset of a [`Span`] into 1-based [`LineColumn`].
    pub fn span_to_line_column(&self, span: Span) -> Option<LineColumn> {
        self.offset_to_line_column(span.source, span.start as usize)
    }

    /// Formats a span into a human-readable diagnostic location string (e.g. `"hello.sm:10:15"`).
    pub fn format_location(&self, span: Span) -> Option<String> {
        let file = self.get(span.source)?;
        let loc = file.offset_to_line_column(span.start as usize)?;
        Some(format!("{}:{}:{}", file.name(), loc.line, loc.column))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_file_creation() {
        let file = SourceFile::new(SourceId(0), "hello.sm", "fn main() {}");
        assert_eq!(file.id(), SourceId(0));
        assert_eq!(file.name(), "hello.sm");
        assert_eq!(file.text(), "fn main() {}");
        assert_eq!(file.len(), 12);
        assert!(!file.is_empty());
        assert_eq!(file.line_count(), 1);
    }

    #[test]
    fn test_sourcemap_registration_and_sequential_ids() {
        let mut map = SourceMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);

        let id0 = map.add("first.sm", "let a = 1;");
        let id1 = map.add("second.sm", "let b = 2;");
        let id2 = map.add("third.sm", "let c = 3;");

        assert_eq!(id0, SourceId(0));
        assert_eq!(id1, SourceId(1));
        assert_eq!(id2, SourceId(2));
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_sourcemap_retrieval() {
        let mut map = SourceMap::new();
        let id = map.add("test.sm", "const X = 42;");

        assert!(map.get(id).is_some());
        assert_eq!(map.get_name(id), Some("test.sm"));
        assert_eq!(map.get_text(id), Some("const X = 42;"));

        // Unknown SourceId returns None
        assert!(map.get(SourceId(999)).is_none());
        assert_eq!(map.get_name(SourceId(999)), None);
        assert_eq!(map.get_text(SourceId(999)), None);
    }

    #[test]
    fn test_single_line_offset_to_line_column() {
        let file = SourceFile::new(SourceId(0), "single.sm", "let x = 10;");

        // Offset 0 -> 'l' -> line 1, column 1
        assert_eq!(file.offset_to_line_column(0), Some(LineColumn::new(1, 1)));
        // Offset 4 -> 'x' -> line 1, column 5
        assert_eq!(file.offset_to_line_column(4), Some(LineColumn::new(1, 5)));
        // Offset 11 -> ';' -> line 1, column 12
        assert_eq!(file.offset_to_line_column(11), Some(LineColumn::new(1, 12)));
    }

    #[test]
    fn test_multiline_unix_newline() {
        let source = "line1\nline2\nline3";
        let file = SourceFile::new(SourceId(0), "unix.sm", source);

        assert_eq!(file.line_count(), 3);

        // line1: bytes 0..5 ('l', 'i', 'n', 'e', '1')
        assert_eq!(file.offset_to_line_column(0), Some(LineColumn::new(1, 1)));
        assert_eq!(file.offset_to_line_column(4), Some(LineColumn::new(1, 5)));

        // byte 5 is '\n' on line 1, column 6
        assert_eq!(file.offset_to_line_column(5), Some(LineColumn::new(1, 6)));

        // byte 6 is 'l' of line2
        assert_eq!(file.offset_to_line_column(6), Some(LineColumn::new(2, 1)));
        assert_eq!(file.offset_to_line_column(10), Some(LineColumn::new(2, 5)));

        // byte 12 is 'l' of line3
        assert_eq!(file.offset_to_line_column(12), Some(LineColumn::new(3, 1)));
    }

    #[test]
    fn test_windows_crlf_newline() {
        let source = "abc\r\ndef\r\nxyz";
        let file = SourceFile::new(SourceId(0), "windows.sm", source);

        // Must not count CRLF as two separate lines
        assert_eq!(file.line_count(), 3);

        // Line 1: "abc\r\n" (bytes 0..5)
        assert_eq!(file.offset_to_line_column(0), Some(LineColumn::new(1, 1))); // 'a'
        assert_eq!(file.offset_to_line_column(2), Some(LineColumn::new(1, 3))); // 'c'
        assert_eq!(file.offset_to_line_column(3), Some(LineColumn::new(1, 4))); // '\r'
        assert_eq!(file.offset_to_line_column(4), Some(LineColumn::new(1, 5))); // '\n'

        // Line 2: "def\r\n" starts at byte 5
        assert_eq!(file.offset_to_line_column(5), Some(LineColumn::new(2, 1))); // 'd'
        assert_eq!(file.offset_to_line_column(7), Some(LineColumn::new(2, 3))); // 'f'

        // Line 3: "xyz" starts at byte 10
        assert_eq!(file.offset_to_line_column(10), Some(LineColumn::new(3, 1))); // 'x'
    }

    #[test]
    fn test_utf8_source_handling() {
        // "fn main() { print(\"বাংলা\"); }"
        // 'ব' is 3 bytes (E0 A6 AC)
        // 'া' is 3 bytes (E0 A6 BE)
        // 'ং' is 3 bytes (E0 A6 82)
        // 'ল' is 3 bytes (E0 A6 B2)
        // 'া' is 3 bytes (E0 A6 BE)
        let source = "print(\"বাংলা\");";
        let file = SourceFile::new(SourceId(0), "utf8.sm", source);

        // 'p' is at byte 0 -> col 1
        assert_eq!(file.offset_to_line_column(0), Some(LineColumn::new(1, 1)));
        // '"' is at byte 6 -> col 7
        assert_eq!(file.offset_to_line_column(6), Some(LineColumn::new(1, 7)));

        // 'ব' starts at byte 7 -> col 8
        assert_eq!(file.offset_to_line_column(7), Some(LineColumn::new(1, 8)));
        // 'া' starts at byte 10 -> col 9
        assert_eq!(file.offset_to_line_column(10), Some(LineColumn::new(1, 9)));
        // 'ং' starts at byte 13 -> col 10
        assert_eq!(file.offset_to_line_column(13), Some(LineColumn::new(1, 10)));
        // 'ল' starts at byte 16 -> col 11
        assert_eq!(file.offset_to_line_column(16), Some(LineColumn::new(1, 11)));
        // 'া' starts at byte 19 -> col 12
        assert_eq!(file.offset_to_line_column(19), Some(LineColumn::new(1, 12)));

        // Closing '"' is at byte 22 -> col 13
        assert_eq!(file.offset_to_line_column(22), Some(LineColumn::new(1, 13)));
    }

    #[test]
    fn test_out_of_range_offset_handling() {
        let file = SourceFile::new(SourceId(0), "test.sm", "hello");
        assert_eq!(file.len(), 5);

        // In range: 0..=5
        assert!(file.offset_to_line_column(0).is_some());
        assert!(file.offset_to_line_column(5).is_some());

        // Out of range: offset 6 > 5
        assert_eq!(file.offset_to_line_column(6), None);
        assert_eq!(file.offset_to_line_column(100), None);
    }

    #[test]
    fn test_sourcemap_format_location() {
        let mut map = SourceMap::new();
        let id = map.add("hello.sm", "fn main() {\n    print(\"Hello, SUMER!\")\n}");

        // Start of print: line 2, col 5
        // "fn main() {\n" is 12 bytes
        // "    print" starts after 4 spaces at offset 12 + 4 = 16
        let span = Span::new(id, 16, 21);
        let loc = map.format_location(span);
        assert_eq!(loc, Some("hello.sm:2:5".to_string()));
    }

    #[test]
    fn test_hello_sm_manual_verification() {
        // Section 16 Manual Verification:
        // fn main() {
        //     print("Hello, SUMER!")
        // }
        let text = "fn main() {\n    print(\"Hello, SUMER!\")\n}";
        let mut map = SourceMap::new();
        let id = map.add("hello.sm", text);

        let file = map.get(id).expect("file should exist");

        // Line 1: 'f' at byte 0 -> line 1, column 1
        assert_eq!(file.offset_to_line_column(0), Some(LineColumn::new(1, 1)));

        // Line 1: "main" at byte 3 -> line 1, column 4
        assert_eq!(file.offset_to_line_column(3), Some(LineColumn::new(1, 4)));

        // Line 2: "print" at byte 16 -> line 2, column 5
        let print_offset = 16;
        let print_span = Span::new(id, print_offset, print_offset + 5);
        assert_eq!(
            map.span_to_line_column(print_span),
            Some(LineColumn::new(2, 5))
        );

        // Line 2: string literal argument start at byte 22 -> line 2, column 11
        let str_offset = 22;
        assert_eq!(
            file.offset_to_line_column(str_offset),
            Some(LineColumn::new(2, 11))
        );

        // Line 3: '}' at byte 43 -> line 3, column 1
        let brace_offset = text.find('}').unwrap();
        assert_eq!(
            file.offset_to_line_column(brace_offset),
            Some(LineColumn::new(3, 1))
        );
    }
}
