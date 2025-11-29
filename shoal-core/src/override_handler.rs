use std::collections::HashMap;

use crate::types::{docker_service::DockerService, stack::Stack, stack_override::StackOverride};

use tracing::debug;

pub fn extract_override(input: &str, stacks: &HashMap<String, Stack>) -> (String, Option<String>) {
    let parts: Vec<&str> = input.split('.').collect();

    for i in (1..=parts.len()).rev() {
        let potential_stack = parts[..i].join(".");
        if stacks.contains_key(&potential_stack) {
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

pub fn apply_overrides(
    docker_services: &mut HashMap<String, DockerService>,
    override_config: &StackOverride,
) {
    debug!("Applying service overrides");
    for (service_name, service) in docker_services {
        if let Some(service_override) = override_config.overrides.get(service_name) {
            debug!("Overriding service: {}", service_name);

            apply_env_override(service, service_override);
            apply_ports_override(service, service_override);
            apply_command_override(service, service_override);
            apply_entrypoint_override(service, service_override);
            apply_volumes_override(service, service_override);
        }
    }
}

fn apply_env_override(
    service: &mut DockerService,
    service_override: &crate::types::stack_override::Override,
) {
    if let Some(env) = &service_override.env {
        let service_env = service.environment.clone().unwrap_or_default();
        let merged_env = merge_hashmaps(&service_env, env);
        debug!("  environment: {} variables set/overridden", env.len());
        for (key, value) in env {
            debug!("    {}={}", key, value);
        }
        service.environment = Some(merged_env);
    }
}

fn apply_ports_override(
    service: &mut DockerService,
    service_override: &crate::types::stack_override::Override,
) {
    if let Some(ports) = &service_override.ports {
        let mut service_ports = service
            .ports
            .as_ref()
            .map(|v| v.to_vec())
            .unwrap_or_default();

        debug!("  ports: {} port(s) set/overridden", ports.len());
        for port_str in ports {
            debug!("    {}", port_str);

            let internal_port = if port_str.contains(':') {
                port_str.split(':').nth(1).unwrap_or(port_str).to_string()
            } else {
                port_str.clone()
            };

            if let Some(existing) = service_ports.iter_mut().find(|p| **p == internal_port) {
                debug!("      (replaced existing port mapping)");
                *existing = port_str.clone();
            } else {
                debug!("      (added new port mapping)");
                service_ports.push(port_str.clone());
            }
        }
        service.ports = Some(service_ports);
    }
}

fn apply_command_override(
    service: &mut DockerService,
    service_override: &crate::types::stack_override::Override,
) {
    if let Some(command) = &service_override.command {
        debug!("  command: {:?}", command);
        service.command = Some(command.clone());
    }
}

fn apply_entrypoint_override(
    service: &mut DockerService,
    service_override: &crate::types::stack_override::Override,
) {
    if let Some(entrypoint) = &service_override.entrypoint {
        debug!("  entrypoint: {:?}", entrypoint);
        service.entrypoint = Some(entrypoint.clone());
    }
}

fn apply_volumes_override(
    service: &mut DockerService,
    service_override: &crate::types::stack_override::Override,
) {
    if let Some(volumes) = &service_override.volumes {
        let mut service_volumes = service
            .volumes
            .as_ref()
            .map(|v| v.to_vec())
            .unwrap_or_default();

        debug!("  volumes: {} volume(s) added", volumes.len());
        for volume in volumes {
            debug!("    {}", volume);
        }

        service_volumes.append(&mut volumes.clone());
        service.volumes = Some(service_volumes);
    }
}

fn merge_hashmaps(
    a: &HashMap<String, String>,
    b: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut result = a.clone();
    result.extend(b.iter().map(|(k, v)| (k.clone(), v.clone())));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::stack::Stack;

    #[test]
    fn test_extract_override_exact_match() {
        let mut stacks = HashMap::new();
        stacks.insert(
            "my-stack".to_string(),
            Stack {
                name: "my-stack".to_string(),
                description: "Test stack".to_string(),
                services: vec![],
            },
        );

        let (stack_name, override_name) = extract_override("my-stack", &stacks);
        assert_eq!(stack_name, "my-stack");
        assert_eq!(override_name, None);
    }

    #[test]
    fn test_extract_override_with_override() {
        let mut stacks = HashMap::new();
        stacks.insert(
            "my-stack".to_string(),
            Stack {
                name: "my-stack".to_string(),
                description: "Test stack".to_string(),
                services: vec![],
            },
        );

        let (stack_name, override_name) = extract_override("my-stack.dev", &stacks);
        assert_eq!(stack_name, "my-stack");
        assert_eq!(override_name, Some("dev".to_string()));
    }

    #[test]
    fn test_extract_override_nested() {
        let mut stacks = HashMap::new();
        stacks.insert(
            "my".to_string(),
            Stack {
                name: "my".to_string(),
                description: "Test stack".to_string(),
                services: vec![],
            },
        );
        stacks.insert(
            "my.stack".to_string(),
            Stack {
                name: "my.stack".to_string(),
                description: "Test stack".to_string(),
                services: vec![],
            },
        );

        // Should match the longest stack name
        let (stack_name, override_name) = extract_override("my.stack.dev", &stacks);
        assert_eq!(stack_name, "my.stack");
        assert_eq!(override_name, Some("dev".to_string()));
    }

    #[test]
    fn test_merge_hashmaps() {
        let mut a = HashMap::new();
        a.insert("KEY1".to_string(), "value1".to_string());
        a.insert("KEY2".to_string(), "value2".to_string());

        let mut b = HashMap::new();
        b.insert("KEY2".to_string(), "new_value2".to_string());
        b.insert("KEY3".to_string(), "value3".to_string());

        let merged = merge_hashmaps(&a, &b);
        assert_eq!(merged.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(merged.get("KEY2"), Some(&"new_value2".to_string())); // b overrides a
        assert_eq!(merged.get("KEY3"), Some(&"value3".to_string()));
        assert_eq!(merged.len(), 3);
    }
}
