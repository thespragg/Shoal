use std::{path::PathBuf, process::Command};

use anyhow::{Result, anyhow};
use tracing::{error, info};

pub struct ComposeManager {
    compose_file_path: PathBuf,
    project_name: String,
}

impl ComposeManager {
    pub fn new(compose_path: impl Into<PathBuf>, project_name: impl Into<String>) -> Self {
        Self {
            compose_file_path: compose_path.into(),
            project_name: project_name.into(),
        }
    }

    pub fn up(&self) -> Result<()> {
        info!("Starting container stack.");

        let mut command = Command::new("docker");
        command
            .arg("compose")
            .arg("--project-name")
            .arg(&self.project_name)
            .arg("-f")
            .arg(&self.compose_file_path)
            .arg("up")
            .arg("-d")
            .arg("--build")
            .arg("--wait");

        let status = command.status()?;

        if status.success() {
            info!("Started all containers successfully.");
            Ok(())
        } else {
            error!("Failed to start containers.");
            Err(anyhow!(
                "Failed to start containers: exit code {:?}",
                status.code()
            ))
        }
    }

    pub fn down(&self) -> Result<()> {
        let mut command = Command::new("docker");
        command
            .arg("compose")
            .arg("--project-name")
            .arg(&self.project_name)
            .arg("-f")
            .arg(&self.compose_file_path)
            .arg("down");

        let status = command.status()?;

        if status.success() {
            info!("Stopped all containers successfully.");
            Ok(())
        } else {
            error!("Failed to stop containers.");
            Err(anyhow!(
                "Failed to stop containers: exit code {:?}",
                status.code()
            ))
        }
    }
}
