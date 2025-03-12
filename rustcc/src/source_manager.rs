use crate::source_file::SourceFile;
use elsa::FrozenMap;
use std::{collections::HashMap, fmt::Debug, fs};

/// This trait defines the interface for a source manager
/// which is responsible for loading source files
/// and caching them
pub trait SourceManager<'a> {
    // TODO: Instead of optional return a Result
    fn load_file<S: Into<&'a str>>(&self, path: S) -> Option<&SourceFile>;
}

/// This class manages all the source files with access to the real filesystem
#[derive(Default)]
pub struct RealFSSourceManager {
    source_files: FrozenMap<String, Box<SourceFile>>,
}

impl RealFSSourceManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            source_files: FrozenMap::new(),
        }
    }

    fn load_file_from_disk(&self, path: &str) -> bool {
        debug_assert!(!self.is_file_loaded(path), "File already loaded");

        if let Ok(content) = fs::read_to_string(path) {
            // Cache the file
            self.source_files.insert(
                path.to_owned(),
                Box::from(SourceFile::new(path.to_owned(), content)),
            );

            return true;
        }

        false
    }

    fn is_file_loaded(&self, path: &str) -> bool {
        self.source_files.get(path).is_some()
    }

    fn get_source_file(&self, path: &str) -> &SourceFile {
        #[expect(clippy::expect_used)]
        self.source_files.get(path).expect("File not found")
    }
}

impl<'a> SourceManager<'a> for RealFSSourceManager {
    fn load_file<S: Into<&'a str>>(&self, path: S) -> Option<&SourceFile> {
        let path = path.into();

        if self.is_file_loaded(path) || self.load_file_from_disk(path) {
            return Some(self.get_source_file(path));
        }

        None
    }
}

/// Source manager which has no access to the real filesystem and allows storing virtual files in virtual
#[derive(Debug, Clone, Default)]
pub struct VirtualSourceManager {
    source_files: HashMap<String, SourceFile>,
}

impl VirtualSourceManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            source_files: HashMap::new(),
        }
    }

    pub fn add_file<S1: Into<String> + Clone, S2: Into<String>>(&mut self, path: S1, content: S2) {
        self.source_files
            .insert(path.clone().into(), SourceFile::new(path, content));
    }
}

impl<'a> SourceManager<'a> for VirtualSourceManager {
    fn load_file<S: Into<&'a str>>(&self, path: S) -> Option<&SourceFile> {
        self.source_files.get(path.into())
    }
}

/// Source manager which doesn't actually manage any files and always fails to load any files.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptySourceManager;

impl EmptySourceManager {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl<'a> SourceManager<'a> for EmptySourceManager {
    fn load_file<S: Into<&'a str>>(&self, _path: S) -> Option<&SourceFile> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_real_fs_source_manager() {
        let source_manager = RealFSSourceManager::new();

        // Create temp file
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test");
        let mut file = fs::File::create(&file_path).unwrap();
        write!(file, "content").unwrap();

        let file_path_string = file_path.into_os_string().into_string().unwrap();

        // Load the file
        let source_file = source_manager.load_file(file_path_string.as_str()).unwrap();

        assert_eq!(source_file.content, "content");

        // Load the same file again (which should now be cached)
        assert!(
            source_manager
                .load_file(file_path_string.as_str())
                .is_some()
        );
    }

    #[test]
    fn test_virtual_source_manager() {
        let mut source_manager = VirtualSourceManager::new();

        source_manager.add_file("test", "content");

        // Load the file
        let source_file = source_manager.load_file("test").unwrap();

        assert_eq!(source_file.content, "content");
    }

    #[test]
    fn test_empty_source_manager() {
        let source_manager = EmptySourceManager;

        assert!(source_manager.load_file("any_path").is_none());
    }
}
