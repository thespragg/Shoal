use std::collections::HashMap;

use crate::{
    compose::ComposeFileManager,
    docker::{orchestrator::ComposeManager, service::build_docker_service},
    override_handler::{apply_overrides, extract_override},
    traits::{CommandExecutor, FileSystem, PathProvider},
    types::{service::Service, stack::Stack, stack_override::StackOverride},
};

use anyhow::{Result, anyhow, bail};
use tracing::{debug, error, info};

pub struct StackManager<FS: FileSystem, PP: PathProvider> {
    services: HashMap<String, Service>,
    stacks: HashMap<String, Stack>,
    overrides: HashMap<String, StackOverride>,
    compose_file_manager: ComposeFileManager<FS, PP>,
    command_executor: std::sync::Arc<dyn CommandExecutor>,
}

impl<FS: FileSystem, PP: PathProvider> StackManager<FS, PP> {
    pub fn new(
        services: HashMap<String, Service>,
        stacks: HashMap<String, Stack>,
        overrides: HashMap<String, StackOverride>,
        compose_file_manager: ComposeFileManager<FS, PP>,
        command_executor: std::sync::Arc<dyn CommandExecutor>,
    ) -> Self {
        StackManager {
            services,
            stacks,
            overrides,
            compose_file_manager,
            command_executor,
        }
    }

    pub fn up(&self, stack_name: impl Into<String>) -> Result<()> {
        let (stack_name, override_name) =
            extract_override(stack_name.into().as_str(), &self.stacks);

        let stack = self.stacks.get(&stack_name).ok_or_else(|| {
            anyhow::anyhow!("Failed to find a stack with the name '{stack_name}'.")
        })?;

        let active_override = if let Some(o) = override_name {
            let found_override = self
                .overrides
                .get(&format!("{}-{}", &stack_name, &o))
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Failed to find an override for {stack_name} with the name '{o}'."
                    )
                })?
                .clone();

            info!(
                "Override {o} is being used. To see what changes this makes to the stack, run up using verbose mode (-v|--verbose)."
            );

            Some(found_override)
        } else {
            None
        };

        self.validate_stack_services(&stack_name, stack)?;

        debug!("Finding docker services for stack {:?}.", stack.services);

        let network_name = format!("{}-network", stack_name.clone());

        let mut docker_services: HashMap<String, _> = stack
            .services
            .iter()
            .map(|service_name| {
                let service = self
                    .services
                    .get(service_name)
                    .expect("Service should exist (validated above)");
                (
                    service_name.clone(),
                    build_docker_service(service, &stack_name, &network_name),
                )
            })
            .collect();

        if let Some(o) = active_override {
            apply_overrides(&mut docker_services, &o);
        }

        let compose_path = self.compose_file_manager.ensure_compose_path(&stack_name)?;
        self.compose_file_manager.generate_compose_file(
            &network_name,
            docker_services,
            &compose_path,
        )?;

        let compose_manager =
            ComposeManager::new(compose_path, stack_name, self.command_executor.clone());
        compose_manager.up()?;

        Ok(())
    }

    pub fn down(&self, stack_name: impl Into<String>) -> Result<()> {
        let stack_name = stack_name.into();
        let compose_path = self.compose_file_manager.compose_file_path(&stack_name)?;
        if !self.compose_file_manager.file_exists(&compose_path) {
            bail!("Stack {stack_name} is not running; compose file missing at {compose_path:?}");
        }

        let compose_manager =
            ComposeManager::new(compose_path, stack_name, self.command_executor.clone());
        compose_manager.down()?;
        Ok(())
    }

    fn validate_stack_services(&self, stack_name: &str, stack: &Stack) -> Result<()> {
        let missing: Vec<&String> = stack
            .services
            .iter()
            .filter(|service_name| !self.services.contains_key(*service_name))
            .collect();

        if !missing.is_empty() {
            error!("Stack '{stack_name}' references non-existent services: {missing:?}");
            return Err(anyhow!(
                "Stack '{stack_name}' references non-existent services: {missing:?}"
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::mocks::{MockCommandExecutor, MockFileSystem, MockPathProvider};
    use crate::types::service::{LocationType, Service, ServiceLocation};
    use std::sync::Arc;

    fn create_test_service(name: &str) -> Service {
        Service {
            service_name: name.to_string(),
            source: ServiceLocation {
                r#type: LocationType::Image,
                location: "test/image:latest".to_string(),
            },
            internal_ports: vec!["8080".to_string()],
        }
    }

    fn create_test_stack(name: &str, services: Vec<String>) -> Stack {
        Stack {
            name: name.to_string(),
            description: "Test stack".to_string(),
            services,
        }
    }

    #[test]
    fn test_validate_stack_services_success() {
        let mut services = HashMap::new();
        services.insert("service1".to_string(), create_test_service("service1"));
        services.insert("service2".to_string(), create_test_service("service2"));

        let mut stacks = HashMap::new();
        stacks.insert(
            "test-stack".to_string(),
            create_test_stack(
                "test-stack",
                vec!["service1".to_string(), "service2".to_string()],
            ),
        );

        let overrides = HashMap::new();
        let file_system = MockFileSystem::new();
        let path_provider = MockPathProvider::new();
        let compose_file_manager = ComposeFileManager::new(file_system, path_provider);
        let command_executor = Arc::new(MockCommandExecutor::new());

        let manager = StackManager::new(
            services,
            stacks,
            overrides,
            compose_file_manager,
            command_executor,
        );

        let stack = manager.stacks.get("test-stack").unwrap();
        let result = manager.validate_stack_services("test-stack", stack);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_stack_services_failure() {
        let mut services = HashMap::new();
        services.insert("service1".to_string(), create_test_service("service1"));

        let mut stacks = HashMap::new();
        stacks.insert(
            "test-stack".to_string(),
            create_test_stack(
                "test-stack",
                vec!["service1".to_string(), "missing-service".to_string()],
            ),
        );

        let overrides = HashMap::new();
        let file_system = MockFileSystem::new();
        let path_provider = MockPathProvider::new();
        let compose_file_manager = ComposeFileManager::new(file_system, path_provider);
        let command_executor = Arc::new(MockCommandExecutor::new());

        let manager = StackManager::new(
            services,
            stacks,
            overrides,
            compose_file_manager,
            command_executor,
        );

        let stack = manager.stacks.get("test-stack").unwrap();
        let result = manager.validate_stack_services("test-stack", stack);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing-service"));
    }
}
