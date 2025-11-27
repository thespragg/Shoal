use crate::{
    config::loader::{load_services, load_stacks},
    manager::ShoalManager,
};
use anyhow::Result;

mod config;
mod docker;
mod manager;
mod types;

pub fn create_shoal_manager() -> Result<ShoalManager> {
    let services = load_services()?;
    let stacks = load_stacks()?;

    Ok(ShoalManager::new(services, stacks))
}
