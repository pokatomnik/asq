use clap::Args;

use crate::{controllers::controller::Controller, services::memory::Memory};

#[derive(Args, Clone, Debug)]
pub(crate) struct MemoriesAddController;

impl Controller for MemoriesAddController {
    fn handle(&self) -> anyhow::Result<()> {
        let memory_to_save = dialoguer::Input::new()
            .with_prompt("What must be saved?")
            .allow_empty(false)
            .interact()?;
        Memory::add_memory(memory_to_save)?;
        println!("Memory saved");
        Ok(())
    }
}
