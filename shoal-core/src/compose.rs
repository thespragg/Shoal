use std::{collections::HashMap, path::PathBuf};

use crate::traits::{FileSystem, PathProvider};
use crate::types::{
    docker_network::DockerNetwork, docker_service::DockerComposeFile, docker_service::DockerService,
};

use anyhow::Result;
use tracing::debug;

pub struct ComposeFileManager<FS: FileSystem, PP: PathProvider> {
    file_system: FS,
    path_provider: PP,
}

impl<FS: FileSystem, PP: PathProvider> ComposeFileManager<FS, PP> {
    pub fn new(file_system: FS, path_provider: PP) -> Self {
        Self {
            file_system,
            path_provider,
        }
    }

    pub fn generate_compose_file(
        &self,
        network_name: &str,
        docker_services: HashMap<String, DockerService>,
        compose_path: &PathBuf,
    ) -> Result<()> {
        let mut networks = HashMap::new();
        networks.insert(network_name.to_string(), Some(DockerNetwork::new(network_name.to_string())));
        
        debug!("Generating docker compose object.");
        let compose = DockerComposeFile {
            services: docker_services,
            networks,
        };

        debug!("Compose object generated, saving to file.");
        let compose_yaml = serde_saphyr::to_string(&compose)?;
        self.file_system.write_file(compose_path, &compose_yaml)?;
        debug!("Compose saved to {:?}", compose_path);

        Ok(())
    }

    pub fn ensure_compose_path(&self, stack_name: &str) -> Result<PathBuf> {
        let stack_dir = self.stack_dir(stack_name)?;
        if !self.file_system.exists(&stack_dir) {
            self.file_system.create_dir_all(&stack_dir)?;
        }

        Ok(stack_dir.join("docker-compose.generated.yml"))
    }

    pub fn compose_file_path(&self, stack_name: &str) -> Result<PathBuf> {
        Ok(self.stack_dir(stack_name)?.join("docker-compose.generated.yml"))
    }

    pub fn file_exists(&self, path: &PathBuf) -> bool {
        self.file_system.exists(path)
    }

    fn stack_dir(&self, stack_name: &str) -> Result<PathBuf> {
        let base_dir = self.path_provider.data_local_dir()?
            .join("shoal")
            .join("stacks")
            .join(stack_name);

        Ok(base_dir)
    }
}

