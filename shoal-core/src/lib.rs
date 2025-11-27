use crate::{config::loader::load_services, manager::ShoalManager};
use anyhow::Result;

mod config;
mod docker;
mod manager;
mod types;

pub fn create_shoal_manager() -> Result<ShoalManager> {
    let services = load_services()?;

    Ok(ShoalManager::new(services))
}
