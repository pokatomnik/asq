use crate::{controllers::controller::Controller, services::memory::Memory};
use clap::Args;

#[derive(Args, Clone, Debug)]
pub(crate) struct MemoriesListController;

impl Controller for MemoriesListController {
    fn handle(&self) -> anyhow::Result<()> {
        let memories = Memory::get_memories();
        if memories.is_empty() {
            println!("No memories saved");
            return Ok(());
        }

        println!("There are user memories:");
        for memory in memories {
            println!("- {memory}")
        }

        Ok(())
    }
}
