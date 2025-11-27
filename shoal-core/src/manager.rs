use std::collections::HashMap;

use crate::{
    docker::compose::build_docker_service,
    types::{
        docker_network::DockerNetwork,
        docker_service::{ComposeWrapper},
        service::Service,
    },
};

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

    pub fn up(&self) {
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

        println!("{}", serde_saphyr::to_string(&compose).unwrap());
    }
}
