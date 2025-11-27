use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockerNetwork {
    pub name: String,
}

impl DockerNetwork{
    pub fn new(name: String) -> DockerNetwork{
        DockerNetwork {
                name,
            }
    }
}