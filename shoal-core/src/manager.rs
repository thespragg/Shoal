use std::{collections::HashMap, env, fs};

use crate::{
    docker::compose::build_docker_service,
    types::{docker_network::DockerNetwork, docker_service::ComposeWrapper, service::Service},
};

use anyhow::Result;

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

    pub fn up(&self) -> Result<()> {
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

        let compose = ComposeWrapper {
            services: docker_services,
            networks: [self.network.clone()]
                .iter()
                .map(|n| (n.name.clone(), None))
                .collect(),
        };

        let temp_dir = env::temp_dir().join("shoal");
        if !temp_dir.exists() {
            fs::create_dir(&temp_dir)?;
        }

        let compose_dir = temp_dir.join("current");
        if !compose_dir.exists() {
            fs::create_dir(&compose_dir)?;
        }

        fs::write(
            compose_dir.join("docker-compose.generated.yml"),
            serde_saphyr::to_string(&compose).unwrap(),
        )?;

        Ok(())
    }
}
