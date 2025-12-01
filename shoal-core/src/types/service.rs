use std::fmt;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub enum LocationType {
    Image,
    Local,
}

impl fmt::Display for LocationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LocationType::Image => "Image",
                LocationType::Local => "Local",
            }
        )
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServiceLocation {
    pub r#type: LocationType,
    pub image: String,
}

impl fmt::Display for ServiceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.r#type, self.image)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Service {
    pub name: String,
    pub source: ServiceLocation,
    pub internal_ports: Vec<String>,
    pub dependencies: Option<Vec<String>>,
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] ports: {:?}",
            self.name, self.source, self.internal_ports
        )
    }
}
