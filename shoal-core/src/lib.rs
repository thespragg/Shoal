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

pub fn create_shoal_manager() -> Result<ShoalManager> {
    let file_system = StdFileSystem;
    let path_provider = StdPathProvider;
    let config_loader = ConfigLoader::new(file_system, path_provider);

    let services = config_loader.load_services()?;
    let stacks = config_loader.load_stacks()?;
    let overrides = config_loader.load_overrides()?;

    Ok(ShoalManager::new(services, stacks, overrides))
}
