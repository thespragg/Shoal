use std::{collections::HashMap, fs, path::PathBuf};

use crate::{
    docker::{orchestrator::ComposeManager, service::build_docker_service},
    types::{
        docker_network::DockerNetwork, docker_service::DockerComposeFile, service::Service,
        stack::Stack, stack_override::StackOverride,
    },
};

use anyhow::{Result, anyhow, bail};
use serde::ser;
use tracing::{debug, error, info};

pub struct ShoalManager {
    services: HashMap<String, Service>,
    stacks: HashMap<String, Stack>,
    overrides: HashMap<String, StackOverride>,
}

impl ShoalManager {
    pub fn new(
        services: HashMap<String, Service>,
        stacks: HashMap<String, Stack>,
        overrides: HashMap<String, StackOverride>,
    ) -> Self {
        ShoalManager {
            services,
            stacks,
            overrides,
        }
    }

    pub fn up(&self, stack_name: impl Into<String>) -> Result<()> {
        let (stack_name, stack_override) = self.extract_override(stack_name.into().as_str());

        let stack = self.stacks.get(&stack_name).ok_or_else(|| {
            anyhow::anyhow!("Failed to find a stack with the name '{}'.", stack_name)
        })?;

        let mut active_override: Option<StackOverride> = None;
        if let Some(o) = stack_override {
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

            active_override = Some(found_override);
        }

        // TODO:: Need to get dependency tree and flatten for proper services list

        debug!("Finding docker services for stack {:?}.", stack.services);

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
            debug!("Applying service overrides");
            for (service_name, service) in &mut docker_services {
                if let Some(service_override) = o.overrides.get(service_name) {
                    debug!("Overriding service: {}", service_name);

                    if let Some(env) = &service_override.env {
                        let service_env = service.environment.clone().unwrap_or_else(HashMap::new);

                        let merged_env = merge_hashmaps(&service_env, env);
                        debug!("  environment: {} variables set/overridden", env.len());
                        for (key, value) in env {
                            debug!("    {}={}", key, value);
                        }

                        service.environment = Some(merged_env);
                    }

                    if let Some(ports) = &service_override.ports {
                        let mut service_ports = service
                            .ports
                            .as_ref()
                            .map(|v| v.to_vec())
                            .unwrap_or_else(Vec::new);

                        debug!("  ports: {} port(s) set/overridden", ports.len());
                        for port_str in ports {
                            debug!("    {}", port_str);

                            let internal_port = if port_str.contains(':') {
                                port_str.split(':').nth(1).unwrap_or(port_str).to_string()
                            } else {
                                port_str.clone()
                            };

                            if let Some(existing) =
                                service_ports.iter_mut().find(|p| **p == internal_port)
                            {
                                debug!("      (replaced existing port mapping)");
                                *existing = port_str.clone();
                            } else {
                                debug!("      (added new port mapping)");
                                service_ports.push(port_str.clone());
                            }
                        }
                        service.ports = Some(service_ports);
                    }

                    if let Some(command) = &service_override.command {
                        debug!("  command: {:?}", command);
                        service.command = Some(command.clone());
                    }

                    if let Some(entrypoint) = &service_override.entrypoint {
                        debug!("  entrypoint: {:?}", entrypoint);
                        service.entrypoint = Some(entrypoint.clone());
                    }

                    if let Some(volumes) = &service_override.volumes {
                        let mut service_volumes = service
                            .volumes
                            .as_ref()
                            .map(|v| v.to_vec())
                            .unwrap_or_else(Vec::new);

                        debug!("  volumes: {} volume(s) added", volumes.len());
                        for volume in volumes {
                            debug!("    {}", volume);
                        }

                        service_volumes.append(&mut volumes.clone());
                        service.volumes = Some(service_volumes);
                    }
                }
            }
        }

        let mut networks = HashMap::new();
        networks.insert(network_name.clone(), Some(DockerNetwork::new(network_name)));
        debug!("Generating docker compose object.");
        let compose = DockerComposeFile {
            services: docker_services,
            networks,
        };

        debug!("Compose object generated, saving to file.");
        let compose_path = ensure_compose_path(&stack_name)?;
        let compose_yaml = serde_saphyr::to_string(&compose)?;
        fs::write(&compose_path, compose_yaml)?;
        debug!("Compose saved to {:?}", compose_path);

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

    fn extract_override(&self, input: &str) -> (String, Option<String>) {
        let parts: Vec<&str> = input.split('.').collect();

        for i in (1..=parts.len()).rev() {
            let potential_stack = parts[..i].join(".");
            if self.stacks.contains_key(&potential_stack) {
                if i < parts.len() {
                    let override_name = parts[i..].join(".");
                    return (potential_stack, Some(override_name));
                } else {
                    return (potential_stack, None);
                }
            }
        }

        (input.to_string(), None)
    }
}

fn ensure_compose_path(stack_name: &str) -> Result<PathBuf> {
    let stack_dir = stack_dir(stack_name)?;
    if !stack_dir.exists() {
        fs::create_dir_all(&stack_dir)?;
    }

    Ok(stack_dir.join("docker-compose.generated.yml"))
}

fn compose_file_path(stack_name: &str) -> Result<PathBuf> {
    Ok(stack_dir(stack_name)?.join("docker-compose.generated.yml"))
}

fn stack_dir(stack_name: &str) -> Result<PathBuf> {
    let base_dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow!("Could not determine local data directory"))?
        .join("shoal")
        .join("stacks")
        .join(stack_name);

    Ok(base_dir)
}

fn merge_hashmaps(
    a: &HashMap<String, String>,
    b: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut result = a.clone();
    result.extend(b.iter().map(|(k, v)| (k.clone(), v.clone())));
    result
}
