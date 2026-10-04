use std::path::Path;

/// Filesystem operations used by application services.
///
/// Kept intentionally small: only the operations that application services need
/// directly. Process spawning and network access have their own ports.
pub trait Filesystem: Send + Sync {
    /// Read a file to a string.
    fn read_to_string(&self, path: &Path) -> std::io::Result<String>;

    /// Write a string to a file, creating parent directories as needed.
    fn write_string(&self, path: &Path, contents: &str) -> std::io::Result<()>;

    /// Report whether a path exists.
    fn exists(&self, path: &Path) -> bool;

    /// Create a directory and all missing parents.
    fn create_dir_all(&self, path: &Path) -> std::io::Result<()>;
}

/// Production filesystem backed by `std::fs`.
#[derive(Debug, Default, Clone, Copy)]
pub struct RealFilesystem;

impl Filesystem for RealFilesystem {
    fn read_to_string(&self, path: &Path) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn write_string(&self, path: &Path, contents: &str) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, contents)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn create_dir_all(&self, path: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(path)
    }
}
