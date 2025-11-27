use std::{collections::HashMap, fs, path::PathBuf};

use crate::types::{
    docker_network::DockerNetwork, docker_service::DockerComposeFile, docker_service::DockerService,
};

use anyhow::{Result, anyhow};
use tracing::debug;

pub fn generate_compose_file(
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
    fs::write(compose_path, compose_yaml)?;
    debug!("Compose saved to {:?}", compose_path);

    Ok(())
}

pub fn ensure_compose_path(stack_name: &str) -> Result<PathBuf> {
    let stack_dir = stack_dir(stack_name)?;
    if !stack_dir.exists() {
        fs::create_dir_all(&stack_dir)?;
    }

    Ok(stack_dir.join("docker-compose.generated.yml"))
}

pub fn compose_file_path(stack_name: &str) -> Result<PathBuf> {
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

