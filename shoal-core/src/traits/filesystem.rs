use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

pub trait FileSystem: Send + Sync {
    fn read_file(&self, path: &Path) -> Result<String>;
    fn write_file(&self, path: &Path, content: &str) -> Result<()>;
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    fn exists(&self, path: &Path) -> bool;
}

pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn read_file(&self, path: &Path) -> Result<String> {
        std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read file {}: {}", path.display(), e))
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        std::fs::write(path, content)
            .map_err(|e| anyhow!("Failed to write file {}: {}", path.display(), e))
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        std::fs::read_dir(path)
            .map_err(|e| anyhow!("Failed to read directory {}: {}", path.display(), e))?
            .map(|entry| {
                entry
                    .map_err(|e| anyhow!("Failed to read directory entry: {}", e))
                    .map(|e| e.path())
            })
            .collect()
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        std::fs::create_dir_all(path)
            .map_err(|e| anyhow!("Failed to create directory {}: {}", path.display(), e))
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}
