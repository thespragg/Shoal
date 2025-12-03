use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use tracing::info;

use crate::traits::CommandExecutor;

pub struct ComposeManager {
    compose_file_path: PathBuf,
    project_name: String,
    command_executor: Arc<dyn CommandExecutor>,
}

impl ComposeManager {
    pub fn new(
        compose_path: impl Into<PathBuf>,
        project_name: impl Into<String>,
        command_executor: Arc<dyn CommandExecutor>,
    ) -> Self {
        Self {
            compose_file_path: compose_path.into(),
            project_name: project_name.into(),
            command_executor,
        }
    }

    pub fn up(&self) -> Result<()> {
        info!("Starting container stack.");

        let compose_path_str = self.compose_file_path.to_string_lossy().to_string();
        let args = [
            "compose",
            "--project-name",
            &self.project_name,
            "-f",
            &compose_path_str,
            "up",
            "-d",
            "--build",
            "--wait",
        ];

        self.command_executor.execute("docker", &args)?;
        info!("Started all containers successfully.");
        Ok(())
    }

    pub fn down(&self) -> Result<()> {
        let compose_path_str = self.compose_file_path.to_string_lossy().to_string();
        let args = [
            "compose",
            "--project-name",
            &self.project_name,
            "-f",
            &compose_path_str,
            "down",
        ];

        self.command_executor.execute("docker", &args)?;
        info!("Stopped all containers successfully.");
        Ok(())
    }
}
