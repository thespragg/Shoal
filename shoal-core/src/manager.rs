use std::{collections::HashMap, env, fs};

use crate::{
    docker::{orchestrator::ComposeManager, service::build_docker_service},
    types::{docker_network::DockerNetwork, docker_service::DockerComposeFile, service::Service},
};

use anyhow::{Result, anyhow};
use tracing::debug;

pub struct ShoalManager {
    network: DockerNetwork,
    services: Vec<Service>,
}

impl ShoalManager {
    pub fn new(services: Vec<Service>) -> ShoalManager {
        ShoalManager {
            services,
            network: DockerNetwork::new("TestNetwork".to_string()),
        }
    }

    pub fn up(&self, stack_name: impl Into<String>) -> Result<()> {
        debug!("Finding docker services.");
        let docker_services = self
            .services
            .iter()
            .map(|service| {
                (
                    service.service_name.clone(),
                    build_docker_service(service, self.network.name.clone()),
                )
            })
            .collect::<HashMap<_, _>>();

        debug!("Generating docker compose object.");
        let compose = DockerComposeFile {
            services: docker_services,
            networks: [self.network.clone()]
                .iter()
                .map(|n| (n.name.clone(), None))
                .collect(),
        };

        debug!("Compose object generated, saving to file.");
        let compose_path = get_or_create_compose_dir(stack_name)?;
        fs::write(&compose_path, serde_saphyr::to_string(&compose).unwrap())?;

        let compose_manager = ComposeManager::new(compose_path);
        compose_manager.up()?;

        Ok(())
    }

    pub fn down(&self, stack_name: impl Into<String>) -> Result<()> {
        let compose_path = get_or_create_compose_dir(stack_name)?;

        let compose_manager = ComposeManager::new(compose_path);
        compose_manager.down()?;
        Ok(())
    }
}

fn get_or_create_compose_dir(stack_name: impl Into<String>) -> Result<String> {
    let temp_dir = env::temp_dir().join("shoal");
    if !temp_dir.exists() {
        fs::create_dir(&temp_dir)?;
    }

    let compose_dir = temp_dir.join("stacks");
    if !compose_dir.exists() {
        fs::create_dir(&compose_dir)?;
    }

    let stack_folder = compose_dir.join(stack_name.into());
    if !stack_folder.exists() {
        fs::create_dir(&stack_folder)?;
    }
    
    let compose_file = stack_folder.join("docker-compose.generated.yml");
    compose_file
        .into_os_string()
        .into_string()
        .map_err(|path| anyhow!("compose path not valid UTF-8: {:?}", path))
}
