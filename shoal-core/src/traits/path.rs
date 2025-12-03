use anyhow::{Result, anyhow};
use std::path::PathBuf;

pub trait PathProvider: Send + Sync {
    fn current_dir(&self) -> Result<PathBuf>;
    fn home_dir(&self) -> Result<PathBuf>;
    fn data_local_dir(&self) -> Result<PathBuf>;
}

pub struct StdPathProvider;

impl PathProvider for StdPathProvider {
    fn current_dir(&self) -> Result<PathBuf> {
        std::env::current_dir().map_err(|e| anyhow!("Failed to get current directory: {e}"))
    }

    fn home_dir(&self) -> Result<PathBuf> {
        dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))
    }

    fn data_local_dir(&self) -> Result<PathBuf> {
        dirs::data_local_dir().ok_or_else(|| anyhow!("Could not determine local data directory"))
    }
}
