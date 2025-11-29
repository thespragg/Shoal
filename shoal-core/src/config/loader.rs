use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use std::{
    collections::HashMap,
    path::PathBuf,
};
use tracing::{debug, warn};

use crate::traits::{FileSystem, PathProvider};
use crate::types::{service::Service, stack::Stack, stack_override::StackOverride};

pub struct ConfigLoader<FS: FileSystem, PP: PathProvider> {
    file_system: FS,
    path_provider: PP,
}

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

impl<FS: FileSystem, PP: PathProvider> ConfigLoader<FS, PP> {
    pub fn new(file_system: FS, path_provider: PP) -> Self {
        Self {
            file_system,
            path_provider,
        }
    }

    pub fn load_overrides(&self) -> Result<HashMap<String, StackOverride>> {
        let local_path = self.path_provider.current_dir()?.join("overrides");
        let global_path = self.path_provider.home_dir()?.join(".shoal/overrides");

        let search_paths: [(FileScope, PathBuf); 2] = [
            (FileScope::Global, global_path),
            (FileScope::Local, local_path),
        ];

        self.load_items(
            &search_paths,
            "Overrides",
            "overrides",
            "Stack override detected.",
            |stack_override: &StackOverride| format!("{}-{}", &stack_override.stack, &stack_override.name),
        )
    }

    pub fn load_stacks(&self) -> Result<HashMap<String, Stack>> {
        let local_path = self.path_provider.current_dir()?.join("stacks");
        let global_path = self.path_provider.home_dir()?.join(".shoal/stacks");

        let search_paths: [(FileScope, PathBuf); 2] = [
            (FileScope::Global, global_path),
            (FileScope::Local, local_path),
        ];

        self.load_items(
            &search_paths,
            "Stacks",
            "stack",
            "Local version of stack detected; using local definition.",
            |stack: &Stack| stack.name.clone(),
        )
    }

    pub fn load_services(&self) -> Result<HashMap<String, Service>> {
        let local_path = self.path_provider.current_dir()?.join("services");
        let global_path = self.path_provider.home_dir()?.join(".shoal/services");

        let search_paths: [(FileScope, PathBuf); 2] = [
            (FileScope::Global, global_path),
            (FileScope::Local, local_path),
        ];

        self.load_items(
            &search_paths,
            "Services",
            "service",
            "Service override detected; using local definition.",
            |service: &Service| service.service_name.clone(),
        )
    }
}

impl<FS: FileSystem, PP: PathProvider> ConfigLoader<FS, PP> {
    fn load_items<T, F>(
        &self,
        search_paths: &[(FileScope, PathBuf)],
        folder_label: &'static str,
        item_label: &'static str,
        override_message: &'static str,
        name_extractor: F,
    ) -> Result<HashMap<String, T>>
    where
        T: DeserializeOwned,
        F: Fn(&T) -> String,
    {
        for (scope, path) in search_paths {
            if self.file_system.exists(path) {
                debug!(?path, %scope, "{} folder exists", folder_label);
            } else {
                debug!(?path, %scope, "{} folder does not exist", folder_label);
            }
        }

        let mut items_by_name: HashMap<String, (FileScope, T)> = HashMap::new();

        for (scope, path) in search_paths {
            if !self.file_system.exists(path) {
                continue;
            }

            for (file_path, contents) in self.read_yaml_files_in_directory(path)? {
                let item: T = serde_saphyr::from_str(&contents).with_context(|| {
                    format!(
                        "Failed to parse {} file: {}",
                        item_label,
                        file_path.display()
                    )
                })?;
                let name = name_extractor(&item);

                if let Some((previous_scope, _)) = items_by_name.insert(name.clone(), (*scope, item)) {
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

        let items: HashMap<String, T> = items_by_name
            .into_iter()
            .map(|(name, (_, item))| (name, item))
            .collect();

        debug!("Identified {} unique {} files", items.len(), item_label);

        Ok(items)
    }

    fn read_yaml_files_in_directory(&self, path: &PathBuf) -> Result<Vec<(PathBuf, String)>> {
        let entries = self.file_system.read_dir(path)
            .with_context(|| format!("Failed to read directory: {}", path.display()))?;

        let mut result = Vec::new();
        for file_path in entries {
            // Check if it's a YAML file
            let is_yaml = file_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e == "yaml" || e == "yml")
                .unwrap_or(false);

            if is_yaml {
                match self.file_system.read_file(&file_path) {
                    Ok(contents) => result.push((file_path, contents)),
                    Err(e) => {
                        warn!("Skipping invalid file {}: {}", file_path.display(), e);
                    }
                }
            }
        }

        Ok(result)
    }
}
