use crate::types::{docker_service::DockerService, service::Service};

/// Build a docker service definition scoped to the provided stack name so that
/// container names remain unique per stack.
pub fn build_docker_service(service: &Service, stack_name: &str, network: String) -> DockerService {
    DockerService {
        container_name: format!("{}-{}", stack_name, service.service_name),
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
