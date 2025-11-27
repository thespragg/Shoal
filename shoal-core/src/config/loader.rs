use anyhow::{Context, Result};
use tracing::debug;
use std::{
    env, fs::{self}, path::Path
};

use crate::types::service::Service;

pub fn load_services() -> Result<Vec<Service>> {
    let local_path = env::current_dir()?.join("services");
    let global_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".shoal/services");

    let paths = [local_path, global_path];

    for path in &paths {
        if path.exists() {
            debug!("Services folder exists at: {:?}", path);
        } else {
            debug!("Services folder does not exist at: {:?}", path);
        }
    }

    let services: Vec<Service> = paths
        .iter()
        .filter(|path| path.exists())
        .map(|path| read_yaml_files_in_directory(path))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .map(|contents| serde_saphyr::from_str(&contents).map_err(anyhow::Error::from))
        .collect::<Result<Vec<Service>, _>>()?;

    debug!("Identified {} service files", services.len());

    for service in &services {
        debug!("{}", service);
    }

    Ok(services)
}


fn read_yaml_files_in_directory(path: &Path) -> Result<Vec<String>> {
    let files: Result<Vec<String>> = fs::read_dir(path)
        .with_context(|| format!("Failed to read directory: {}", path.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e == "yaml")
                .unwrap_or(false)
        })
        .map(|path| -> Result<String> {
         fs::read_to_string(&path)
                .with_context(|| format!("Failed to read file: {}", path.display()))
        })
        .collect();
    
    files
}