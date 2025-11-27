use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct StackOverride {
    pub name: String,
    pub stack: String,
    description: String,
    pub overrides: HashMap<String, Override>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Override {
    pub env: Option<HashMap<String, String>>,
    pub ports: Option<Vec<String>>,
    pub command: Option<Vec<String>>,
    pub entrypoint: Option<Vec<String>>,
    pub volumes: Option<Vec<String>>,
}
