use serde::Serialize;

use crate::utils::tool_definition::tsd_function_parameters::TSDFunctionParameters;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct TSDFunction<T: Serialize> {
    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "description")]
    description: String,

    #[serde(rename = "parameters")]
    parameters: TSDFunctionParameters<T>,
}

impl<T: Serialize> TSDFunction<T> {
    pub fn new(name: impl AsRef<str>, description: impl AsRef<str>, properties: T) -> Self {
        let parameters = TSDFunctionParameters::new(properties);
        Self {
            name: name.as_ref().to_string(),
            description: description.as_ref().to_string(),
            parameters,
        }
    }
}
