use crate::services::{memory::Memory, pipe_processor::pipe_operator::PipeOperator};

/// This operator is used by the LLM to store information in long-term memory.
/// It should not be used in user prompts.
pub(crate) struct Remember;

impl Remember {
    pub fn new() -> Self {
        Self {}
    }
}

impl PipeOperator for Remember {
    fn handle(&self, source: &str) -> anyhow::Result<String> {
        Memory::add_memory(source.trim().to_string())?;
        Ok(String::default())
    }
}
