use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use std::{
    collections::HashMap,
    env,
    fs::{self},
    path::{Path, PathBuf},
};
use tracing::{debug, warn};

use crate::types::{service::Service, stack::Stack};

#[derive(Clone, Copy, Debug)]
enum FileScope {
    Local,
    Global,
}

impl std::fmt::Display for FileScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileScope::Local => write!(f, "local"),
            FileScope::Global => write!(f, "global"),
        }
    }
}

pub fn load_stacks() -> Result<Vec<Stack>> {
    let local_path = env::current_dir()?.join("stacks");
    let global_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".shoal/stacks");

    let search_paths: [(FileScope, PathBuf); 2] = [
        (FileScope::Global, global_path),
        (FileScope::Local, local_path),
    ];

    load_items(
        &search_paths,
        "Stacks",
        "stack",
        "Stack override detected; using local definition.",
        |stack: &Stack| stack.name.clone(),
    )
}

pub fn load_services() -> Result<Vec<Service>> {
    let local_path = env::current_dir()?.join("services");
    let global_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".shoal/services");

    let search_paths: [(FileScope, PathBuf); 2] = [
        (FileScope::Global, global_path),
        (FileScope::Local, local_path),
    ];

    load_items(
        &search_paths,
        "Services",
        "service",
        "Service override detected; using local definition.",
        |service: &Service| service.service_name.clone(),
    )
}

fn load_items<T, F>(
    search_paths: &[(FileScope, PathBuf)],
    folder_label: &'static str,
    item_label: &'static str,
    override_message: &'static str,
    name_extractor: F,
) -> Result<Vec<T>>
where
    T: DeserializeOwned,
    F: Fn(&T) -> String,
{
    for (scope, path) in search_paths {
        if path.exists() {
            debug!(?path, %scope, "{} folder exists", folder_label);
        } else {
            debug!(?path, %scope, "{} folder does not exist", folder_label);
        }
    }

    let mut items_by_name: HashMap<String, (FileScope, T)> = HashMap::new();

    for (scope, path) in search_paths {
        if !path.exists() {
            continue;
        }

        for contents in read_yaml_files_in_directory(path)? {
            let item: T =
                serde_saphyr::from_str(&contents).map_err(anyhow::Error::from)?;
            let name = name_extractor(&item);

            if let Some((previous_scope, _)) =
                items_by_name.insert(name.clone(), (*scope, item))
            {
                warn!(
                    service = %name,
                    previous = %previous_scope,
                    current = %scope,
                    "{}", override_message
                );
            } else {
                debug!(service = %name, source = %scope, "Loaded {}", item_label);
            }
        }
    }

    let items: Vec<T> = items_by_name
        .into_iter()
        .map(|(_, (_, item))| item)
        .collect();

    debug!("Identified {} unique {} files", items.len(), item_label);

    Ok(items)
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
                .map(|e| e == "yaml" || e == "yml")
                .unwrap_or(false)
        })
        .map(|path| -> Result<String> {
            fs::read_to_string(&path)
                .with_context(|| format!("Failed to read file: {}", path.display()))
        })
        .collect();

    files
}
