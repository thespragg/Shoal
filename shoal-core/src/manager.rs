use std::collections::HashMap;

use crate::{
    stack::StackManager,
    types::{service::Service, stack::Stack, stack_override::StackOverride},
};

use anyhow::Result;

pub struct ShoalManager {
    stack_manager: StackManager,
}

impl ShoalManager {
    pub fn new(
        services: HashMap<String, Service>,
        stacks: HashMap<String, Stack>,
        overrides: HashMap<String, StackOverride>,
    ) -> Self {
        ShoalManager {
            stack_manager: StackManager::new(services, stacks, overrides),
        }
    }

    pub fn up(&self, stack_name: impl Into<String>) -> Result<()> {
        self.stack_manager.up(stack_name)
    }

    pub fn down(&self, stack_name: impl Into<String>) -> Result<()> {
        self.stack_manager.down(stack_name)
    }
}
