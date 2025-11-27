use anyhow::{Context, Result};
use tracing::{debug, warn};
use std::{
    collections::HashMap, env, fs::{self}, path::Path
};

use crate::types::service::Service;

#[derive(Clone, Copy, Debug)]
enum ServiceScope {
    Local,
    Global,
}

impl std::fmt::Display for ServiceScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceScope::Local => write!(f, "local"),
            ServiceScope::Global => write!(f, "global"),
        }
    }
}

pub fn load_services() -> Result<Vec<Service>> {
    let local_path = env::current_dir()?.join("services");
    let global_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".shoal/services");

    let search_paths = [
        (ServiceScope::Global, global_path),
        (ServiceScope::Local, local_path),
    ];

    for (scope, path) in &search_paths {
        if path.exists() {
            debug!(?path, %scope, "Services folder exists");
        } else {
            debug!(?path, %scope, "Services folder does not exist");
        }
    }

    let mut services_by_name: HashMap<String, (ServiceScope, Service)> = HashMap::new();

    for (scope, path) in &search_paths {
        if !path.exists() {
            continue;
        }

        for contents in read_yaml_files_in_directory(path)? {
            let service: Service = serde_saphyr::from_str(&contents).map_err(anyhow::Error::from)?;
            let name = service.service_name.clone();

            if let Some((previous_scope, _)) = services_by_name.insert(name.clone(), (*scope, service)) {
                warn!(
                    service = %name,
                    previous = %previous_scope,
                    current = %scope,
                    "Service override detected; using local definition."
                );
            } else {
                debug!(service = %name, source = %scope, "Loaded service");
            }
        }
    }

    let services: Vec<Service> = services_by_name
        .into_iter()
        .map(|(_, (_, service))| service)
        .collect();

    debug!("Identified {} unique service files", services.len());

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