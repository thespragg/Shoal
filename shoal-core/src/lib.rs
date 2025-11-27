use crate::{config::loader::load_services, manager::ShoalManager};
use anyhow::Result;

mod config;
mod manager;
mod types;
mod docker;

pub fn create_shoal_manager() -> Result<ShoalManager> {
    let services = load_services()?;

    Ok(ShoalManager::new(services))
}
