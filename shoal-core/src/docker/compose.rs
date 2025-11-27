use crate::types::{docker_service::DockerService, service::Service};

pub fn build_docker_service(service: &Service, network: String) -> DockerService {
    DockerService {
        container_name: service.service_name.clone(),
        image: Some(service.source.location.clone()),
        ports: Some(service.internal_ports.clone()),
        networks: Some([network].to_vec()),
        build_context: None,
        dockerfile: None,
        entrypoint: None,
        command: None,
        environment: None,
        volumes: None,
        depends_on: None,
        restart: None,
    }
}
