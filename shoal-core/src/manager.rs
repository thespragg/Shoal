use std::{collections::HashMap, sync::Arc};

use crate::{
    compose::ComposeFileManager,
    stack::StackManager,
    traits::{StdCommandExecutor, StdFileSystem, StdPathProvider},
    types::{service::Service, stack::Stack, stack_override::StackOverride},
};

use anyhow::Result;

pub struct ShoalManager {
    stack_manager: StackManager<StdFileSystem, StdPathProvider>,
}

impl ShoalManager {
    pub fn new(
        services: HashMap<String, Service>,
        stacks: HashMap<String, Stack>,
        overrides: HashMap<String, StackOverride>,
    ) -> Self {
        let file_system = StdFileSystem;
        let path_provider = StdPathProvider;
        let compose_file_manager = ComposeFileManager::new(file_system, path_provider);
        let command_executor = Arc::new(StdCommandExecutor);

        let stack_manager = StackManager::new(
            services,
            stacks,
            overrides,
            compose_file_manager,
            command_executor,
        );

        ShoalManager { stack_manager }
    }

    pub fn up(&self, stack_name: impl Into<String>) -> Result<()> {
        self.stack_manager.up(stack_name)
    }

    pub fn down(&self, stack_name: impl Into<String>) -> Result<()> {
        self.stack_manager.down(stack_name)
    }
}
