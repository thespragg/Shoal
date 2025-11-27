use std::collections::HashMap;

use crate::{
    compose::{ensure_compose_path, compose_file_path, generate_compose_file},
    docker::{orchestrator::ComposeManager, service::build_docker_service},
    override_handler::{apply_overrides, extract_override},
    types::{service::Service, stack::Stack, stack_override::StackOverride},
};

use anyhow::{Result, anyhow, bail};
use tracing::{debug, error, info};

pub struct StackManager {
    services: HashMap<String, Service>,
    stacks: HashMap<String, Stack>,
    overrides: HashMap<String, StackOverride>,
}

impl StackManager {
    pub fn new(
        services: HashMap<String, Service>,
        stacks: HashMap<String, Stack>,
        overrides: HashMap<String, StackOverride>,
    ) -> Self {
        StackManager {
            services,
            stacks,
            overrides,
        }
    }

    pub fn up(&self, stack_name: impl Into<String>) -> Result<()> {
        let (stack_name, override_name) = extract_override(
            stack_name.into().as_str(),
            &self.stacks,
        );

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

        let compose_path = ensure_compose_path(&stack_name)?;
        generate_compose_file(&network_name, docker_services, &compose_path)?;

        let compose_manager = ComposeManager::new(compose_path, stack_name);
        compose_manager.up()?;

        Ok(())
    }

    pub fn down(&self, stack_name: impl Into<String>) -> Result<()> {
        let stack_name = stack_name.into();
        let compose_path = compose_file_path(&stack_name)?;
        if !compose_path.exists() {
            bail!(
                "Stack {} is not running; compose file missing at {:?}",
                stack_name,
                compose_path
            );
        }

        let compose_manager = ComposeManager::new(compose_path, stack_name);
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
            error!(
                "Stack '{}' references non-existent services: {:?}",
                stack_name, missing
            );
            return Err(anyhow!(
                "Stack '{}' references non-existent services: {:?}",
                stack_name,
                missing
            ));
        }

        Ok(())
    }
}

