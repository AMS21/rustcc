use crate::source_location::SourceLocation;

// TODO: Same problem for PartialOrd and Ord as with SourceLocation

/// A range of source code, represented by a beginning and ending location.
///
/// # Examples
///
/// ```
/// # use rustcc::source_file::SourceFile;
/// # use rustcc::source_location::SourceLocation;
/// # use rustcc::source_range::SourceRange;
///
/// let source_file = SourceFile::new("path/to/file", "content");
/// let begin = SourceLocation::new(&source_file, 0, 1, 1);
/// let end = SourceLocation::new(&source_file, 1, 1, 2);
/// let range = SourceRange::new(begin, end);
///
/// assert!(range.is_valid());
/// assert_eq!(range.begin, begin);
/// assert_eq!(range.end, end);
/// ```
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct SourceRange<'a> {
    pub begin: SourceLocation<'a>,
    pub end: SourceLocation<'a>,
}

impl<'a> SourceRange<'a> {
    /// Creates a new `SourceRange` with the given begin and end locations.
    ///
    /// # Parameters
    ///
    /// - `begin`: The beginning of the range.
    /// - `end`: The end of the range.
    ///
    /// # Returns
    ///
    /// A new `SourceRange` with the given begin and end locations.
    ///
    /// # Panics
    ///
    /// Panics if any of the following conditions are true:
    /// - The begin and end locations are not in the same source file.
    /// - The begin location is after the end location.
    /// - The begin location is on the same line as the end location, but the begin column is
    ///   greater than the end column.
    /// - The begin location is on the same line as the end location, the begin column is the same as
    ///   the end column, but the begin index is greater than the end index.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustcc::source_file::SourceFile;
    /// # use rustcc::source_location::SourceLocation;
    /// # use rustcc::source_range::SourceRange;
    ///
    /// let source_file = SourceFile::new("path/to/file", "content");
    /// let begin = SourceLocation::new(&source_file, 0, 1, 1);
    /// let end = SourceLocation::new(&source_file, 1, 1, 2);
    /// let range = SourceRange::new(begin, end);
    ///
    /// assert!(range.is_valid());
    /// assert_eq!(range.begin, begin);
    /// assert_eq!(range.end, end);
    /// ```
    #[must_use]
    pub fn new(begin: SourceLocation<'a>, end: SourceLocation<'a>) -> Self {
        debug_assert!(
            begin.source_file == end.source_file,
            "Begin and end must be in the same file.\nBegin: {begin}\nEnd:   {end}\nBegin index: {}\nEnd index:   {}",
            begin.index,
            end.index,
        );
        debug_assert!(
            begin.line <= end.line,
            "Begin location must be before end location.\nBegin: {begin}\nEnd:   {end}\nBegin index: {}\nEnd index:   {}",
            begin.index,
            end.index,
        );
        debug_assert!(
            begin.line != end.line || begin.column <= end.column,
            "Begin location must be before end location.\nBegin: {begin}\nEnd:   {end}\nBegin index: {}\nEnd index:   {}",
            begin.index,
            end.index,
        );
        debug_assert!(
            begin.index <= end.index,
            "Begin location must be before end location.\nBegin: {begin}\nEnd:   {end}\nBegin index: {}\nEnd index:   {}",
            begin.index,
            end.index,
        );
        debug_assert!(
            begin.line != end.line || begin.column != end.column || begin.index == end.index,
            "If begin and end are on the same line and are on the same column they must have the same index.\nBegin: {begin}\nEnd:   {end}\nBegin index: {}\nEnd index:   {}",
            begin.index,
            end.index,
        );

        Self { begin, end }
    }

    /// Creates a new `SourceRange` with the given `location` as both the begin and end of the range.
    ///
    /// # Parameters
    ///
    /// - `location`: The location to use as both the begin and end of the range.
    ///
    /// # Returns
    ///
    /// A new `SourceRange` with the given `location` as both the begin and end of the range.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustcc::source_file::SourceFile;
    /// # use rustcc::source_location::SourceLocation;
    /// # use rustcc::source_range::SourceRange;
    ///
    /// let source_file = SourceFile::new("path/to/file", "content");
    /// let location = SourceLocation::new(&source_file, 0, 1, 1);
    /// let range = SourceRange::from_location(location);
    ///
    /// assert!(range.is_valid());
    /// assert_eq!(range.begin, location);
    /// assert_eq!(range.end, location);
    /// ```
    #[must_use]
    pub const fn from_location(location: SourceLocation<'a>) -> Self {
        Self {
            begin: location,
            end: location,
        }
    }

    /// Creates a new invalid `SourceRange`.
    ///
    /// An invalid `SourceRange` has both the begin and end locations set to invalid locations.
    ///
    /// # Returns
    ///
    /// An invalid `SourceRange`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustcc::source_range::SourceRange;
    ///
    /// let range = SourceRange::invalid();
    ///
    /// assert!(!range.is_valid());
    /// assert!(!range.begin.is_valid());
    /// assert!(!range.end.is_valid());
    /// ```
    #[must_use]
    pub const fn invalid() -> Self {
        Self {
            begin: SourceLocation::invalid(),
            end: SourceLocation::invalid(),
        }
    }

    /// Returns true if both the begin and end locations are valid, and false if either of them are
    /// invalid.
    ///
    /// # Returns
    ///
    /// `true` if both the begin and end locations are valid, and `false` if either of them are invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustcc::source_file::SourceFile;
    /// # use rustcc::source_location::SourceLocation;
    /// # use rustcc::source_range::SourceRange;
    ///
    /// // Valid
    /// let source_file = SourceFile::new("path/to/file", "content");
    /// let begin = SourceLocation::new(&source_file, 0, 1, 1);
    /// let end = SourceLocation::new(&source_file, 1, 1, 2);
    /// let range = SourceRange::new(begin, end);
    ///
    /// assert!(range.is_valid());
    /// assert!(range.begin.is_valid());
    /// assert!(range.end.is_valid());
    ///
    /// // Invalid
    /// let range = SourceRange::invalid();
    ///
    /// assert!(!range.is_valid());
    /// assert!(!range.begin.is_valid());
    /// assert!(!range.end.is_valid());
    /// ```
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.begin.is_valid() && self.end.is_valid()
    }

    /// Returns the source text of the range, or `None` if the source file is not available.
    ///
    /// # Returns
    ///
    /// The source text of the range, or `None` if the source file is not available.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustcc::source_file::SourceFile;
    /// # use rustcc::source_location::SourceLocation;
    /// # use rustcc::source_range::SourceRange;
    ///
    /// let content = "Hello, world!";
    /// let source_file = SourceFile::new("path/to/file", content);
    /// let begin = SourceLocation::new(&source_file, 0, 1, 1);
    /// let end = SourceLocation::new(&source_file, 4, 1, 5);
    /// let range = SourceRange::new(begin, end);
    ///
    /// assert_eq!(range.source_text(), Some("Hello"));
    /// ```
    #[must_use]
    pub fn source_text(&self) -> Option<&'a str> {
        let source_file = self.begin.source_file?;

        if self.begin == self.end {
            let character = &source_file.content[self.begin.index..].chars().next()?;
            let end_index = self.begin.index + character.len_utf8();

            return source_file.content.get(self.begin.index..end_index);
        }

        source_file.content.get(self.begin.index..=self.end.index)
    }
}

impl Default for SourceRange<'_> {
    fn default() -> Self {
        Self::invalid()
    }
}

impl<'a> From<SourceLocation<'a>> for SourceRange<'a> {
    fn from(location: SourceLocation<'a>) -> Self {
        Self::from_location(location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_file::SourceFile;

    #[test]
    fn test_new_same_file_valid_range() {
        let source_file = SourceFile::new("path/to/file", "content");
        let begin = SourceLocation::new(&source_file, 0, 1, 1);
        let end = SourceLocation::new(&source_file, 2, 1, 3);
        let range = SourceRange::new(begin, end);

        assert!(range.is_valid());
        assert_eq!(range.begin, begin);
        assert_eq!(range.end, end);
    }

    #[test]
    #[should_panic(expected = "Begin location must be before end location")]
    fn test_new_same_file_invalid_range_begin_after_end() {
        let source_file = SourceFile::new("path/to/file", "content\nmore content");
        let begin = SourceLocation::new(&source_file, 0, 2, 1);
        let end = SourceLocation::new(&source_file, 0, 1, 1);

        let _range = SourceRange::new(begin, end);
    }

    #[test]
    #[should_panic(expected = "Begin location must be before end location")]
    fn test_new_same_file_invalid_range_same_line_begin_column_greater_than_end_column() {
        let source_file = SourceFile::new("path/to/file", "content");
        let begin = SourceLocation::new(&source_file, 0, 1, 2);
        let end = SourceLocation::new(&source_file, 0, 1, 1);

        let _range = SourceRange::new(begin, end);
    }

    #[test]
    #[should_panic(expected = "Begin and end must be in the same file")]
    fn test_new_different_files() {
        let source_file1 = SourceFile::new("path/to/file1", "content1");
        let source_file2 = SourceFile::new("path/to/file2", "content2");
        let begin = SourceLocation::new(&source_file1, 0, 1, 1);
        let end = SourceLocation::new(&source_file2, 1, 1, 2);

        let _range = SourceRange::new(begin, end);
    }

    #[test]
    #[should_panic(expected = "Begin location must be before end location")]
    fn test_new_same_file_invalid_range_same_line_same_column_begin_index_greater_than_end_index() {
        let source_file = SourceFile::new("path/to/file", "content");
        let begin = SourceLocation::new(&source_file, 1, 1, 1);
        let end = SourceLocation::new(&source_file, 0, 1, 1);

        let _range = SourceRange::new(begin, end);
    }

    #[test]
    #[should_panic(
        expected = "If begin and end are on the same line and are on the same column they must have the same index"
    )]
    fn test_new_same_file_invalid_range_same_line_same_column_different_index() {
        let source_file = SourceFile::new("path/to/file", "content");
        let begin = SourceLocation::new(&source_file, 0, 1, 1);
        let end = SourceLocation::new(&source_file, 1, 1, 1);

        let _range = SourceRange::new(begin, end);
    }

    #[test]
    fn test_new_begin_and_end_at_same_location() {
        let source_file = SourceFile::new("path/to/file", "content");
        let location = SourceLocation::new(&source_file, 0, 1, 1);
        let range = SourceRange::new(location, location);

        assert_eq!(range.begin, location);
        assert_eq!(range.end, location);
    }

    #[test]
    fn test_new_from_location_valid_location() {
        let source_file = SourceFile::new("path/to/file", "content");
        let location = SourceLocation::new(&source_file, 1, 1, 2);
        let range = SourceRange::from_location(location);

        assert_eq!(range.begin, location);
        assert_eq!(range.end, location);
    }

    #[test]
    fn test_new_from_location_invalid_location() {
        let location = SourceLocation::invalid();
        let range = SourceRange::from_location(location);

        assert_eq!(range.begin, location);
        assert_eq!(range.end, location);
    }

    #[test]
    fn test_from_location_to_range() {
        let source_file = SourceFile::new("path/to/file", "content");
        let location = SourceLocation::new(&source_file, 0, 1, 2);
        let range = SourceRange::from(location);

        assert_eq!(range.begin, location);
        assert_eq!(range.end, location);
    }

    #[test]
    fn test_from_invalid_location_to_range() {
        let location = SourceLocation::invalid();
        let range = SourceRange::from(location);

        assert_eq!(range.begin, location);
        assert_eq!(range.end, location);
    }

    #[test]
    fn test_invalid_returns_invalid_source_location() {
        let result = SourceRange::invalid();

        assert_eq!(result.begin, SourceLocation::invalid());
        assert_eq!(result.end, SourceLocation::invalid());
    }

    #[test]
    fn test_is_valid() {
        let source_file = SourceFile::new("path/to/file", "content");
        let begin = SourceLocation::new(&source_file, 0, 1, 1);
        let end = SourceLocation::new(&source_file, 1, 1, 2);
        let range = SourceRange::new(begin, end);

        assert!(range.is_valid());
        assert!(range.begin.is_valid());
        assert!(range.end.is_valid());
    }

    #[test]
    fn test_is_valid_invalid() {
        let begin = SourceLocation::invalid();
        let end = SourceLocation::invalid();
        let range = SourceRange::new(begin, end);

        assert!(!range.is_valid());
    }

    #[test]
    fn test_source_text_valid_range() {
        let content = "Hello, world!";
        let source_file = SourceFile::new("path/to/file", content);
        let begin = SourceLocation::new(&source_file, 0, 1, 1);
        let end = SourceLocation::new(&source_file, 4, 1, 5);
        let range = SourceRange::new(begin, end);

        assert_eq!(range.source_text(), Some("Hello"));
    }

    #[test]
    fn test_source_text_empty_range() {
        let content = "Hello, world!";
        let source_file = SourceFile::new("path/to/file", content);
        let location = SourceLocation::new(&source_file, 0, 1, 1);
        let range = SourceRange::new(location, location);

        assert_eq!(range.source_text(), Some("H"));

        let location = SourceLocation::new(&source_file, 1, 1, 2);
        let range = SourceRange::from_location(location);

        assert_eq!(range.source_text(), Some("e"));
    }

    #[test]
    fn test_source_text_utf8() {
        let content = "aこbѤc";
        let source_file = SourceFile::new("path/to/file", content);

        let location = SourceLocation::new(&source_file, 0, 1, 1);
        let range = SourceRange::from_location(location);

        assert_eq!(range.source_text(), Some("a"));

        let location = SourceLocation::new(&source_file, 1, 1, 2);
        let range = SourceRange::from_location(location);

        assert_eq!(range.source_text(), Some("こ"));

        let begin = SourceLocation::new(&source_file, 0, 1, 1);
        let end = SourceLocation::new(&source_file, 3, 1, 2);
        let range = SourceRange::new(begin, end);

        assert_eq!(range.source_text(), Some("aこ"));

        let begin = SourceLocation::new(&source_file, 0, 1, 1);
        let end = SourceLocation::new(&source_file, 4, 1, 3);
        let range = SourceRange::new(begin, end);

        assert_eq!(range.source_text(), Some("aこb"));

        let location = SourceLocation::new(&source_file, 5, 1, 4);
        let range = SourceRange::from_location(location);

        assert_eq!(range.source_text(), Some("Ѥ"));
    }

    #[test]
    fn test_source_text_none_source_file() {
        let begin = SourceLocation::invalid();
        let end = SourceLocation::invalid();
        let range = SourceRange::new(begin, end);

        assert_eq!(range.source_text(), None);
    }
}
