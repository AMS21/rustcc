use crate::{source_file::SourceFile, source_range::SourceRange};
use std::fmt;

// TODO: Maybe custom implementations for PartialOrd and Ord since it makes no sense to compare SourceLocations with different source files
/// A location in a source file, represented by a line and column number.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct SourceLocation<'a> {
    pub source_file: Option<&'a SourceFile>,
    pub index: usize,
    pub line: u32,
    pub column: u32,
}

impl<'a> SourceLocation<'a> {
    /// Creates a new `SourceLocation` with the given source file, line, and column.
    ///
    /// # Parameters
    ///
    /// - `source_file`: A reference to the source file associated with this location.
    /// - `line`: The line number in the source file. Must be greater than 0.
    /// - `column`: The column number in the source file. Must be greater than 0.
    ///
    /// # Panics
    ///
    /// Panics if any of the following conditions are met:
    /// - `line` is 0.
    /// - `column` is 0.
    /// - The line number exceeds the number of lines in the source file.
    /// - The column number exceeds the number of characters in the line.
    /// - The index exceeds the number of characters in the source file.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustcc::source_file::SourceFile;
    /// # use rustcc::source_location::SourceLocation;
    /// let source_file = SourceFile::new("path/to/file", "content");
    /// let location = SourceLocation::new(&source_file, 1, 1, 2);
    ///
    /// assert_eq!(location.source_file, Some(&source_file));
    /// assert_eq!(location.index, 1);
    /// assert_eq!(location.line, 1);
    /// assert_eq!(location.column, 2);
    /// ```
    #[must_use]
    pub fn new(source_file: &'a SourceFile, index: usize, line: u32, column: u32) -> Self {
        debug_assert!(
            line > 0,
            "Line must be greater than 0.\nSource file: '{}'",
            source_file.path
        );
        debug_assert!(
            column > 0,
            "Column must be greater than 0\nSource file: '{}'",
            source_file.path
        );

        let file_lines = source_file.content.lines().count();
        let line_length = source_file
            .content
            .lines()
            .nth((line - 1) as usize)
            .map(|line| line.chars().count());
        let file_chars = source_file.content.len();

        debug_assert!(
            file_lines >= line as usize,
            "Line number exceeds the number of lines in the source file.\nExpected at most {file_lines}, found {line}\nSource file: '{}'",
            source_file.path
        );
        if let Some(line_length) = line_length {
            debug_assert!(
                line_length >= column as usize,
                "Column number exceeds the number of characters in the line.\nExpected at most {line_length}, found {column}.\nSource file: '{}'\nLine: {line}",
                source_file.path
            );
        }
        debug_assert!(
            index < file_chars,
            "Index exceeds the number of characters in the source file.\nExpected at most {file_chars}, found {index}.\nSource file: '{}'",
            source_file.path
        );

        Self {
            source_file: Some(source_file),
            index,
            line,
            column,
        }
    }

    /// Creates a new `SourceLocation` with the given line and column, but without a source file.
    ///
    /// This is used for creating `SourceLocation`s for scratch source files.
    ///
    /// # Parameters
    ///
    /// - `line`: The line number in the source file. Must be greater than 0.
    /// - `column`: The column number in the source file. Must be greater than 0.
    ///
    /// # Panics
    ///
    /// Panics if `line` is 0 or `column` is 0.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustcc::source_location::SourceLocation;
    /// let location = SourceLocation::new_scratch(3, 2);
    ///
    /// assert_eq!(location.source_file, None);
    /// assert_eq!(location.index, 0);
    /// assert_eq!(location.line, 3);
    /// assert_eq!(location.column, 2);
    /// ```
    #[must_use]
    pub fn new_scratch(line: u32, column: u32) -> Self {
        assert!(line > 0, "Line must be greater than 0");
        assert!(column > 0, "Column must be greater than 0");

        Self {
            source_file: None,
            index: 0,
            line,
            column,
        }
    }

    /// Creates an invalid `SourceLocation`.
    ///
    /// # Examples
    /// ```
    /// # use rustcc::source_location::SourceLocation;
    /// let invalid_location = SourceLocation::invalid();
    ///
    /// assert_eq!(invalid_location.source_file, None);
    /// assert_eq!(invalid_location.index, 0);
    /// assert_eq!(invalid_location.line, 0);
    /// assert_eq!(invalid_location.column, 0);
    /// ```
    #[must_use]
    pub const fn invalid() -> Self {
        Self {
            source_file: None,
            index: 0,
            line: 0,
            column: 0,
        }
    }

    /// Returns true if the location is valid, and false if it is invalid.
    ///
    /// # Examples
    /// ```
    /// # use rustcc::source_file::SourceFile;
    /// # use rustcc::source_location::SourceLocation;
    /// // Valid
    /// let source_file = SourceFile::new("path/to/file", "content");
    /// let location = SourceLocation::new(&source_file, 3, 1, 2);
    /// assert_eq!(location.is_valid(), true);
    ///
    /// // Invalid
    /// let invalid_location = SourceLocation::invalid();
    /// assert_eq!(invalid_location.is_valid(), false)
    /// ```
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.line != 0
    }

    /// Returns a `SourceRange` with the same begin and end locations as this `SourceLocation`.
    ///
    /// # Examples
    /// ```
    /// # use rustcc::source_file::SourceFile;
    /// # use rustcc::source_location::SourceLocation;
    /// let source_file = SourceFile::new("path/to/file", "content");
    /// let location = SourceLocation::new(&source_file, 1, 1, 2);
    /// let range = location.to_range();
    ///
    /// assert!(range.is_valid());
    /// assert_eq!(range.begin, location);
    /// assert_eq!(range.end, location);
    /// ```
    #[must_use]
    pub const fn to_range(&self) -> SourceRange {
        SourceRange {
            begin: *self,
            end: *self,
        }
    }

    /// Returns a `SourceRange` with the same begin and end locations as this `SourceLocation` and consumes the location in the process.
    ///
    /// # Examples
    /// ```
    /// # use rustcc::source_file::SourceFile;
    /// # use rustcc::source_location::SourceLocation;
    /// let source_file = SourceFile::new("path/to/file", "content");
    /// let location = SourceLocation::new(&source_file, 1, 1, 2);
    /// let range = location.as_range();
    ///
    /// assert!(range.is_valid());
    /// assert_eq!(range.begin, location);
    /// assert_eq!(range.end, location);
    /// ```
    #[must_use]
    pub const fn as_range(self) -> SourceRange<'a> {
        SourceRange {
            begin: self,
            end: self,
        }
    }
}

impl Default for SourceLocation<'_> {
    fn default() -> Self {
        Self::invalid()
    }
}

impl fmt::Display for SourceLocation<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.is_valid() {
            return write!(formatter, "<invalid>");
        }

        write!(
            formatter,
            "{}:{}:{}",
            self.source_file
                .map_or("<scratch>", |source_file| &source_file.path),
            self.line,
            self.column
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_source_location() {
        let source_file = SourceFile::new("path/to/file", "content");
        let location = SourceLocation::new(&source_file, 1, 1, 2);

        assert_eq!(location.source_file, Some(&source_file));
        assert_eq!(location.index, 1);
        assert_eq!(location.line, 1);
        assert_eq!(location.column, 2);
    }

    #[test]
    #[should_panic(expected = "Line must be greater than 0")]
    fn test_invalid_line_number() {
        let source_file = SourceFile::new("path/to/file", "content");
        let _location = SourceLocation::new(&source_file, 0, 0, 2);
    }

    #[test]
    #[should_panic(expected = "Column must be greater than 0")]
    fn test_invalid_column_number() {
        let source_file = SourceFile::new("path/to/file", "content");
        let _location = SourceLocation::new(&source_file, 0, 3, 0);
    }

    #[test]
    #[should_panic(expected = "Line number exceeds the number of lines in the source file")]
    fn test_line_number_exceeds_lines() {
        let source_file = SourceFile::new("path/to/file", "content");
        let _location = SourceLocation::new(&source_file, 0, 4, 2);
    }

    #[test]
    #[should_panic(expected = "Column number exceeds the number of characters in the line")]
    fn test_column_number_exceeds_characters() {
        let source_file = SourceFile::new("path/to/file", "content");
        let _location = SourceLocation::new(&source_file, 0, 1, 100);
    }

    #[test]
    #[should_panic(expected = "Index exceeds the number of characters in the source file")]
    fn test_index_exceeds_characters() {
        let source_file = SourceFile::new("path/to/file", "content");
        let _location = SourceLocation::new(&source_file, 100, 1, 2);
    }

    #[test]
    fn test_new_scratch_valid() {
        let location = SourceLocation::new_scratch(3, 2);

        assert!(location.source_file.is_none());
        assert_eq!(location.index, 0);
        assert_eq!(location.line, 3);
        assert_eq!(location.column, 2);
    }

    #[test]
    #[should_panic(expected = "Line must be greater than 0")]
    fn test_new_scratch_line_zero() {
        let _location = SourceLocation::new_scratch(0, 2);
    }

    #[test]
    #[should_panic(expected = "Column must be greater than 0")]
    fn test_new_scratch_column_zero() {
        let _location = SourceLocation::new_scratch(3, 0);
    }

    #[test]
    fn test_invalid_source_location() {
        let location = SourceLocation::invalid();

        assert!(location.source_file.is_none());
        assert_eq!(location.index, 0);
        assert_eq!(location.line, 0);
        assert_eq!(location.column, 0);
    }

    #[test]
    fn test_is_valid_valid_location() {
        let source_file = SourceFile::new("path/to/file", "content");
        let location = SourceLocation::new(&source_file, 1, 1, 2);

        assert!(location.is_valid());
    }

    #[test]
    fn test_is_valid_invalid_location() {
        let location = SourceLocation::invalid();

        assert!(!location.is_valid());
    }

    #[test]
    fn test_default() {
        let default_location = SourceLocation::default();

        assert!(!default_location.is_valid());
    }

    #[test]
    fn test_fmt_valid_source_location() {
        let source_file = SourceFile::new("path/to/file", "content");
        let location = SourceLocation::new(&source_file, 0, 1, 2);
        let expected_output = format!("{}:{}:{}", source_file.path, location.line, location.column);

        assert_eq!(expected_output, format!("{location}"));
    }

    #[test]
    fn test_fmt_invalid_source_location() {
        let location = SourceLocation::invalid();
        let expected_output = "<invalid>";

        assert_eq!(expected_output, format!("{location}"));
    }

    #[test]
    fn test_to_range_valid_location() {
        let source_file = SourceFile::new("path/to/file", "content");
        let location = SourceLocation::new(&source_file, 1, 1, 2);
        let range = location.to_range();

        assert!(range.is_valid());
        assert_eq!(range.begin, location);
        assert_eq!(range.end, location);
    }

    #[test]
    fn test_to_range_invalid_location() {
        let location = SourceLocation::invalid();
        let range = location.to_range();

        assert!(!range.is_valid());
        assert_eq!(range.begin, location);
        assert_eq!(range.end, location);
    }

    #[test]
    fn test_to_range_scratch_location() {
        let location = SourceLocation::new_scratch(3, 2);
        let range = location.to_range();

        assert!(range.is_valid());
        assert_eq!(range.begin, location);
        assert_eq!(range.end, location);
    }

    #[test]
    fn test_as_range_valid_location() {
        let source_file = SourceFile::new("path/to/file", "content");
        let location = SourceLocation::new(&source_file, 1, 1, 2);
        let range = location.as_range();

        assert!(range.is_valid());
        assert_eq!(range.begin, location);
        assert_eq!(range.end, location);
    }

    #[test]
    fn test_as_range_invalid_location() {
        let location = SourceLocation::invalid();
        let range = location.as_range();

        assert!(!range.is_valid());
        assert_eq!(range.begin, location);
        assert_eq!(range.end, location);
    }

    #[test]
    fn test_as_range_scratch_location() {
        let location = SourceLocation::new_scratch(3, 2);
        let range = location.as_range();

        assert!(range.is_valid());
        assert_eq!(range.begin, location);
        assert_eq!(range.end, location);
    }
}
