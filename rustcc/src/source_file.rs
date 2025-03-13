/// Represents a source file with a path and its content.
///
/// # Examples
///
/// ```
/// # use rustcc::source_file::SourceFile;
///
/// let source_file = SourceFile::new("test_path.c", "int main() { return 0; }");
///
/// assert_eq!(source_file.path, "test_path.c");
/// assert_eq!(source_file.content, "int main() { return 0; }");
/// ```
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct SourceFile {
    pub path: String,
    pub content: String,
}

impl SourceFile {
    #[must_use]
    pub fn new<P: Into<String>, C: Into<String>>(path: P, content: C) -> Self {
        let path = path.into();

        // Assert that path is a valid path
        assert!(!path.contains("\0"), "Path contains null byte");
        assert!(!path.contains("\n"), "Path contains newline");
        assert!(!path.contains(".."), "Path contains '..'");
        assert!(!path.contains("//"), "Path contains '//'");
        assert!(!path.contains("\\\\"), "Path contains '\\\\'");
        assert!(!path.contains("/*"), "Path contains '/*'");
        assert!(!path.contains("*/"), "Path contains '*/'");

        Self {
            path,
            content: content.into(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    #[test]
    fn test_source_file_creation() {
        let path = "test_path.c";
        let content = "int main() { return 0; }";
        let source_file = SourceFile::new(path, content);

        assert_eq!(source_file.path, path);
        assert_eq!(source_file.content, content);
    }

    #[test]
    fn test_source_file_equality() {
        let path = "test_path.c";
        let content = "int main() { return 0; }";
        let source_file1 = SourceFile::new(path, content);
        let source_file2 = SourceFile::new(path, content);

        assert_eq!(source_file1, source_file2);
    }

    #[test]
    fn test_source_file_inequality() {
        let source_file1 = SourceFile::new("path1.c", "int main() { return 0; }");
        let source_file2 = SourceFile::new("path2.c", "int main() { return 0; }");

        assert_ne!(source_file1, source_file2);
    }

    #[test]
    fn test_source_file_clone() {
        let source_file = SourceFile::new("test_path.c", "int main() { return 0; }");
        let cloned_source_file = source_file.clone();

        assert_eq!(source_file, cloned_source_file);
    }

    #[test]
    fn test_source_file_hash() {
        let source_file = SourceFile::new("test_path.c", "int main() { return 0; }");

        let mut hasher = DefaultHasher::new();
        source_file.hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher = DefaultHasher::new();
        source_file.hash(&mut hasher);
        let hash2 = hasher.finish();

        assert_eq!(hash1, hash2);
    }
}
