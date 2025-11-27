use std::{collections::HashMap, fs, path::PathBuf};

use crate::{
    docker::{orchestrator::ComposeManager, service::build_docker_service},
    types::{
        docker_network::DockerNetwork, docker_service::DockerComposeFile, service::Service,
        stack::Stack,
    },
};

use anyhow::{Result, anyhow, bail};
use tracing::debug;

pub struct ShoalManager {
    network: DockerNetwork,
    services: Vec<Service>,
    stacks: Vec<Stack>,
}

impl ShoalManager {
    pub fn new(services: Vec<Service>, stacks: Vec<Stack>) -> ShoalManager {
        ShoalManager {
            services,
            network: DockerNetwork::new("TestNetwork".to_string()),
            stacks,
        }
    }

    pub fn up(&self, stack_name: impl Into<String>) -> Result<()> {
        let stack_name = stack_name.into();

        let stack = self
            .stacks
            .iter()
            .find(|s| s.name == stack_name)
            .ok_or(anyhow::anyhow!(
                "Failed to find a stack with the name {}.",
                stack_name
            ))?;

        debug!("Finding docker services for stack {:?}.", stack.services);
        let docker_services = self
            .services
            .iter()
            .filter(|s| stack.services.contains(&s.service_name))
            .map(|service| {
                (
                    service.service_name.clone(),
                    build_docker_service(service, &stack_name, self.network.name.clone()),
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
        let compose_path = ensure_compose_path(&stack_name)?;
        let compose_yaml = serde_saphyr::to_string(&compose)?;
        fs::write(&compose_path, compose_yaml)?;

        let compose_manager = ComposeManager::new(compose_path, stack_name);
        compose_manager.up()?;

        Ok(())
    }

    pub fn down(&self, stack_name: impl Into<String>) -> Result<()> {
        let stack_name = stack_name.into();
        let compose_path = compose_file_path(&stack_name)?;
        if !compose_path.exists() {
            bail!(
                "Stack {} is not running; compose file missing at {:?}",
                stack_name,
                compose_path
            );
        }

        let compose_manager = ComposeManager::new(compose_path, stack_name);
        compose_manager.down()?;
        Ok(())
    }
}

fn ensure_compose_path(stack_name: &str) -> Result<PathBuf> {
    let stack_dir = stack_dir(stack_name)?;
    if !stack_dir.exists() {
        fs::create_dir_all(&stack_dir)?;
    }

    Ok(stack_dir.join("docker-compose.generated.yml"))
}

fn compose_file_path(stack_name: &str) -> Result<PathBuf> {
    Ok(stack_dir(stack_name)?.join("docker-compose.generated.yml"))
}

fn stack_dir(stack_name: &str) -> Result<PathBuf> {
    let base_dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow!("Could not determine local data directory"))?
        .join("shoal")
        .join("stacks")
        .join(stack_name);

    Ok(base_dir)
}
