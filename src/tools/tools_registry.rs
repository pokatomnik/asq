use serde_json::Value;

use crate::tools::{memory::memory::Memory, tool::Tool};

pub(crate) struct ToolsRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolsRegistry {
    pub fn new() -> Self {
        let mut tools = Vec::<Box<dyn Tool>>::with_capacity(1);
        tools.push(Box::new(Memory::new()));
        Self {
            tools: vec![Box::new(Memory::new())],
        }
    }

    pub fn tools(&self) -> &[Box<dyn Tool>] {
        &self.tools
    }

    pub fn tool_definitions(&self) -> anyhow::Result<Vec<Value>> {
        let definitions = self
            .tools
            .iter()
            .map(|v| v.definition())
            .filter_map(|d| d.ok())
            .collect::<Vec<Value>>();
        Ok(definitions)
    }

    pub fn call_tool(&self, name: &str, params_raw: &str) -> (Vec<String>, Vec<String>) {
        let mut results = Vec::with_capacity(1);
        let mut err_descriptions = Vec::with_capacity(1);

        for tool in self.tools.iter() {
            if name != tool.name() {
                continue;
            }
            match tool.exec(params_raw) {
                Ok(result) => results.push(result),
                Err(e) => err_descriptions.push(e.to_string()),
            }
        }

        (results, err_descriptions)
    }
}
