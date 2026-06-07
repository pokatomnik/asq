use serde_json::Value;

use crate::tools::{memory::Memory, tool::Tool};

#[derive(Debug)]
pub(crate) struct ToolsRegistry {
    memory: Memory,
}

impl ToolsRegistry {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
        }
    }

    fn call_memory(&self, params_raw: &str, results: &mut Vec<String>, errors: &mut Vec<String>) {
        match self.memory.exec(params_raw) {
            Ok(result) => results.push(result),
            Err(e) => errors.push(e.to_string()),
        }
    }

    pub fn tool_definitions() -> anyhow::Result<Vec<Value>> {
        Ok(vec![Memory::definition()?])
    }

    pub fn call_tool(&self, name: &str, params_raw: &str) -> (Vec<String>, Vec<String>) {
        let mut results = Vec::with_capacity(1);
        let mut err_descriptions = Vec::with_capacity(1);

        if name == Memory::NAME {
            self.call_memory(params_raw, &mut results, &mut err_descriptions);
        }

        (results, err_descriptions)
    }
}
