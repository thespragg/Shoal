use crate::types::{docker_service::DockerService, service::Service};

pub fn build_docker_service(service: &Service, stack_name: &str, network: &str) -> DockerService {
    DockerService {
        container_name: format!("{}-{}", stack_name, service.name),
        image: Some(service.source.image.clone()),
        ports: Some(service.internal_ports.clone()),
        networks: Some(vec![network.to_string()]),
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
