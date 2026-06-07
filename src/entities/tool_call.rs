use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ToolCall {
    #[serde(rename = "id")]
    id: String,

    #[serde(rename = "type")]
    r#type: ToolCallType,

    #[serde(rename = "function")]
    function: Option<ToolCallFunction>,

    #[serde(rename = "extra_content", skip_serializing_if = "Option::is_none")]
    extra_content: Option<Value>,
}

impl ToolCall {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn function(&self) -> Option<&ToolCallFunction> {
        self.function.as_ref()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ToolCallFunction {
    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "arguments")]
    arguments: String,
}

impl ToolCallFunction {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> &str {
        &self.arguments
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub(crate) enum ToolCallType {
    #[serde(rename = "function")]
    Function,
}

impl Display for ToolCallType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolCallType::Function => f.write_str("function"),
        }
    }
}
