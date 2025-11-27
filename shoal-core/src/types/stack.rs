use serde::Deserialize;

#[derive(Deserialize)]
pub struct Stack {
    pub name: String,
    pub description: String,
    pub services: Vec<String>,
}
