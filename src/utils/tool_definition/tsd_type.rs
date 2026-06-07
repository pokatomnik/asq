use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub(crate) enum TDSType {
    #[serde(rename = "function")]
    Function,
}

impl Display for TDSType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TDSType::Function => f.write_str("function"),
        }
    }
}
