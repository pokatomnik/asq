use serde::Serialize;
use serde_json::Value;

use crate::utils::tool_definition::{tsd_function::TSDFunction, tsd_type::TDSType};

/// Struct for schema definition describing
#[derive(Clone, Debug, Serialize)]
pub(crate) struct TSD<T: Serialize> {
    #[serde(rename = "type")]
    r#type: TDSType,

    #[serde(rename = "function")]
    function: TSDFunction<T>,
}

impl<T> TryFrom<TSD<T>> for Value
where
    T: Serialize,
{
    type Error = anyhow::Error;

    fn try_from(value: TSD<T>) -> Result<Self, Self::Error> {
        let result = serde_json::to_value::<TSD<T>>(value)?;
        Ok(result)
    }
}

impl<T: Serialize> TSD<T> {
    pub fn new(name: impl AsRef<str>, description: impl AsRef<str>, properties: T) -> Self {
        let function = TSDFunction::new(name, description, properties);
        Self {
            r#type: TDSType::Function,
            function,
        }
    }
}
