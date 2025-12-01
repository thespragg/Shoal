use std::path::PathBuf;

use crate::{
    config::loader::ConfigLoader,
    manager::ShoalManager,
    traits::{StdFileSystem, StdPathProvider},
};
use anyhow::Result;

mod compose;
mod config;
mod docker;
mod manager;
mod override_handler;
mod stack;
mod traits;
mod types;

pub fn create_shoal_manager(path: &Option<PathBuf>) -> Result<ShoalManager> {
    let file_system = StdFileSystem;
    let path_provider = StdPathProvider;
    let config_loader = ConfigLoader::new(file_system, path_provider);

    let services = config_loader.load_services(path)?;
    let stacks = config_loader.load_stacks(path)?;
    let overrides = config_loader.load_overrides(path)?;

    Ok(ShoalManager::new(services, stacks, overrides))
}
