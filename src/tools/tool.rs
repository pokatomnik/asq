use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(crate) trait Tool<T> {
    /// Get the name of this tool
    fn name() -> &'static str;

    /// Get JSON Schema definition for this tool
    fn schema_def() -> anyhow::Result<Value>;

    /// Execute tool
    /// Execute tool and get `String` result Or error
    fn execute(&self, arg: &T) -> anyhow::Result<String>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum ToolType {
    #[serde(rename = "function")]
    Function,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum ToolParamsType {
    #[serde(rename = "object")]
    Object,
}

impl Display for ToolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolType::Function => f.write_str("function"),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ToolParamsDefinition<T: Serialize> {
    #[serde(rename = "type")]
    r#type: ToolParamsType,

    #[serde(rename = "additionalProperties")]
    additional_properties: bool,

    #[serde(rename = "properties")]
    properties: T,
}

impl<T> ToolParamsDefinition<T>
where
    T: Serialize,
{
    pub fn new(r#type: ToolParamsType, additional_properties: bool, properties: T) -> Self {
        Self {
            r#type,
            additional_properties,
            properties,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct FunctionDefinition<T: Serialize> {
    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "description")]
    description: String,

    #[serde(rename = "parameters")]
    parameters: ToolParamsDefinition<T>,
}

impl<T> FunctionDefinition<T>
where
    T: Serialize,
{
    pub fn new(
        name: impl AsRef<str>,
        description: impl AsRef<str>,
        parameters: ToolParamsDefinition<T>,
    ) -> Self {
        Self {
            name: name.as_ref().to_string(),
            description: description.as_ref().to_string(),
            parameters,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ToolDefinition<T: Serialize> {
    #[serde(rename = "type")]
    r#type: ToolType,

    #[serde(rename = "function")]
    function: FunctionDefinition<T>,
}

impl<T> ToolDefinition<T>
where
    T: Serialize,
{
    pub fn new(r#type: ToolType, function: FunctionDefinition<T>) -> Self {
        Self {
            r#type: r#type,
            function,
        }
    }
}
