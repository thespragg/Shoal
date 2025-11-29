use anyhow::{Result, anyhow};
use std::process::Command;

pub trait CommandExecutor: Send + Sync {
    fn execute(&self, program: &str, args: &[&str]) -> Result<()>;
}

pub struct StdCommandExecutor;

impl CommandExecutor for StdCommandExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<()> {
        let status = Command::new(program).args(args).status()?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!(
                "Command '{}' failed with exit code: {:?}",
                program,
                status.code()
            ))
        }
    }
}
