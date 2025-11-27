use std::fmt;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum LocationType {
    Image,
    Local,
}

impl fmt::Display for LocationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            LocationType::Image => "Image",
            LocationType::Local => "Local",
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct ServiceLocation {
    pub r#type: LocationType,
    pub location: String,
}

impl fmt::Display for ServiceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.r#type, self.location)
    }
}

#[derive(Debug, Deserialize)]
pub struct Service {
    pub service_name: String,
    pub source: ServiceLocation,
    pub internal_ports: Vec<String>,
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] ports: {:?}",
            self.service_name, self.source, self.internal_ports
        )
    }
}
