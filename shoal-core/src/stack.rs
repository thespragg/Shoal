use std::collections::{HashMap, HashSet};

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
            anyhow::anyhow!("Failed to find a stack with the name '{}'.", stack_name)
        })?;

        let active_override = if let Some(o) = override_name {
            let found_override = self
                .overrides
                .get(&format!("{}-{}", &stack_name, &o))
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Failed to find an override for {} with the name '{}'.",
                        stack_name,
                        o
                    )
                })?
                .clone();

            info!(
                "Override {} is being used. To see what changes this makes to the stack, run up using verbose mode (-v|--verbose).",
                o
            );

            Some(found_override)
        } else {
            None
        };

        let stack_services = populate_service_list(&stack.services, &self.services)?;

        let all_service_names = flatten_dependencies(&stack_services);

        let all_services = populate_service_list(&all_service_names, &self.services);

        debug!("Finding docker services for stack {:?}.", stack.services);

        let network_name = format!("{}-network", stack_name.clone());

        let mut docker_services: HashMap<String, _> = all_services?
            .iter()
            .map(|service| {
                (
                    service.name.clone(),
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
            bail!(
                "Stack {} is not running; compose file missing at {:?}",
                stack_name,
                compose_path
            );
        }

        let compose_manager =
            ComposeManager::new(compose_path, stack_name, self.command_executor.clone());
        compose_manager.down()?;
        Ok(())
    }
}

fn flatten_dependencies(services: &[Service]) -> Vec<String> {
    let mut dependencies: Vec<_> = services
        .iter()
        .flat_map(|s| s.dependencies.clone().unwrap_or_else(Vec::new))
        .collect();

    dependencies.extend(services.iter().map(|s| s.name.clone()));

    dependencies
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

fn populate_service_list(
    services: &[String],
    registered_services: &HashMap<String, Service>,
) -> Result<Vec<Service>> {
    use std::collections::HashSet;

    let mut all_services = HashSet::new();
    let mut to_process: Vec<String> = services.to_vec();

    while let Some(service_name) = to_process.pop() {
        if all_services.contains(&service_name) {
            continue;
        }

        let service = registered_services.get(&service_name).ok_or_else(|| {
            error!("Missing required service: {}", service_name);
            anyhow!("Missing required service: {}", service_name)
        })?;

        if let Some(deps) = &service.dependencies {
            to_process.extend(deps.clone());
        }

        all_services.insert(service_name);
    }

    let populated_services: Vec<Service> = all_services
        .iter()
        .map(|s| registered_services.get(s).cloned().unwrap())
        .collect();

    Ok(populated_services)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::service::{LocationType, Service, ServiceLocation};

    fn create_test_service(name: &str) -> Service {
        Service {
            name: name.to_string(),
            source: ServiceLocation {
                r#type: LocationType::Image,
                image: "test/image:latest".to_string(),
            },
            internal_ports: vec!["8080".to_string()],
            dependencies: None,
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

        let result = populate_service_list(&stacks.get("test-stack").unwrap().services, &services);
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

        let result = populate_service_list(&stacks.get("test-stack").unwrap().services, &services);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing-service"));
    }
}
