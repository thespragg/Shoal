use std::process::Command;

use anyhow::Result;
use tracing::{error, info};

pub struct ComposeManager {
    compose_file_path: String,
}

impl ComposeManager {
    pub fn new(compose_path: impl Into<String>) -> Self {
        Self {
            compose_file_path: compose_path.into(),
        }
    }

    pub fn up(&self) -> Result<()> {
        info!("Starting container stack.");

        let status = Command::new("docker")
            .args([
                "compose",
                "-f",
                &self.compose_file_path,
                "up",
                "-d",
                "--build",
                "--wait",
            ])
            .status()?;

        if status.success() {
            info!("Started all containers successfully.");
            Ok(())
        } else {
            error!("Failed to start cointainers.");
            Err(anyhow::anyhow!(
                "Failed to start containers: exit code {:?}",
                status.code()
            ))
        }
    }

    pub fn down(&self) -> Result<()> {
        let status = Command::new("docker")
            .args(["compose", "-f", &self.compose_file_path, "down"])
            .status()?;

        if status.success() {
            info!("Stopped all containers successfully.");
            Ok(())
        } else {
            error!("Failed to stop cointainers.");
            Err(anyhow::anyhow!(
                "Failed to stop containers: exit code {:?}",
                status.code()
            ))
        }
    }
}
