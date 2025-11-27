use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::docker_network::DockerNetwork;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockerComposeFile {
    pub services: HashMap<String, DockerService>,
    pub networks: HashMap<String, Option<DockerNetwork>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockerService {
    pub container_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_context: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dockerfile: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<Vec<(String, String)>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub networks: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<String>,
}
