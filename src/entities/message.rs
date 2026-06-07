use serde::{Deserialize, Serialize};

use crate::entities::role::Role;
use crate::entities::tool_call::ToolCall;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Message {
    #[serde(rename = "role")]
    role: Role,

    #[serde(rename = "contents", alias = "content")]
    contents: Option<String>,

    #[serde(rename = "tool_call_id", skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,

    #[serde(rename = "tool_calls", skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
}

impl Message {
    pub fn new(role: Role, contents: &str) -> Self {
        Self {
            role,
            contents: Some(contents.to_string()),
            tool_call_id: Default::default(),
            tool_calls: Default::default(),
        }
    }

    pub fn assistant_tool_call(contents: Option<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: Role::Assistant,
            contents,
            tool_call_id: Default::default(),
            tool_calls: Some(tool_calls),
        }
    }

    pub fn with_tool_call_id(mut self, tool_call_id: String) -> Self {
        self.tool_call_id = Some(tool_call_id);
        self
    }

    pub fn content(&self) -> Option<&str> {
        self.contents.as_deref()
    }

    pub fn role(&self) -> &Role {
        &self.role
    }

    pub fn tool_call_id(&self) -> Option<&str> {
        self.tool_call_id.as_deref()
    }

    pub fn tool_calls(&self) -> Option<&[ToolCall]> {
        self.tool_calls.as_deref()
    }
}
