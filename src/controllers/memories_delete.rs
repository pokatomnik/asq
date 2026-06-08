use std::collections::HashSet;

use clap::Args;

use crate::{controllers::controller::Controller, services::memory::Memory};

#[derive(Args, Clone, Debug)]
pub(crate) struct MemoriesDeleteController;

impl MemoriesDeleteController {
    fn ask_remove_indexes(&self, memories: &[impl AsRef<str>]) -> HashSet<usize> {
        dialoguer::MultiSelect::new()
            .report(false)
            .clear(true)
            .items(memories.iter().map(|m| m.as_ref()))
            .with_prompt("Select memories to delete")
            .interact()
            .unwrap_or_default()
            .iter()
            .map(ToOwned::to_owned)
            .collect::<HashSet<usize>>()
    }
}

impl Controller for MemoriesDeleteController {
    fn handle(&self) -> anyhow::Result<()> {
        let memories = Memory::get_memories();
        let indexes_to_remove = self.ask_remove_indexes(memories.as_slice());
        if indexes_to_remove.is_empty() {
            println!("No memories to remove");
            return Ok(());
        }
        let updated_memories = memories
            .iter()
            .enumerate()
            .filter_map(|(idx, memory)| match &indexes_to_remove.contains(&idx) {
                true => None,
                false => Some(memory.as_str()),
            })
            .collect::<Vec<&str>>();

        Memory::replace_memories(updated_memories)?;

        println!("Memories removed");

        Ok(())
    }
}
