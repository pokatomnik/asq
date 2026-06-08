use serde::{Deserialize, Serialize};

use crate::services::memory::Memory as MemoryService;
use crate::{tools::tool::Tool, utils::tool_definition::tool_schema_definition::TSD};

#[derive(Debug)]
pub(crate) struct Memory;

impl Memory {
    pub fn new() -> Self {
        Self {}
    }
}

impl Tool for Memory {
    fn name(&self) -> &'static str {
        "memory"
    }

    fn skill(&self) -> &'static str {
        include_str!("./SKILL.md")
    }

    fn definition(&self) -> anyhow::Result<serde_json::Value> {
        let tool_definition = TSD::new(
            self.name(),
            "Persistent memory tool",
            serde_json::json!({
                "text": {
                    "type": "string",
                    "description": "One short factual memory about the user, based only on what the user directly said.",
                    "minLength": 1,
                    "maxLength": 500
                }
            }),
        );

        tool_definition.try_into()
    }

    fn exec(&self, params: &str) -> anyhow::Result<String> {
        let params = serde_json::from_str::<MemoryParams>(params)?;
        MemoryService::add_memory(params.text.as_str())?;
        Ok(format!("Memory \"{}\" saved", params.text.as_str()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MemoryParams {
    #[serde(rename = "text")]
    text: String,
}
