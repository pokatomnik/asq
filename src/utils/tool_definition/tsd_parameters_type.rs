use std::fmt::Display;

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) enum TSDParametersType {
    #[serde(rename = "object")]
    Object,
}

impl Display for TSDParametersType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TSDParametersType::Object => f.write_str("object"),
        }
    }
}
