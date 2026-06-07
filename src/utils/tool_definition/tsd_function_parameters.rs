use serde::Serialize;

use crate::utils::tool_definition::tsd_parameters_type::TSDParametersType;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct TSDFunctionParameters<T: Serialize> {
    #[serde(rename = "type")]
    r#type: TSDParametersType,

    #[serde(rename = "additionalProperties")]
    additional_properties: bool,

    #[serde(rename = "properties")]
    properties: T,
}

impl<T: Serialize> TSDFunctionParameters<T> {
    pub fn new(properties: T) -> Self {
        Self {
            r#type: TSDParametersType::Object,
            additional_properties: false,
            properties,
        }
    }
}
