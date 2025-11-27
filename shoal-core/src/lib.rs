use crate::{
    config::loader::{load_overrides, load_services, load_stacks},
    manager::ShoalManager,
};
use anyhow::Result;

mod compose;
mod config;
mod docker;
mod manager;
mod override_handler;
mod stack;
mod types;

pub fn create_shoal_manager() -> Result<ShoalManager> {
    let services = load_services()?;
    let stacks = load_stacks()?;
    let overrides = load_overrides()?;

    Ok(ShoalManager::new(services, stacks, overrides))
}
