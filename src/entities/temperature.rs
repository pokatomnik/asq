use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub(crate) enum Temperature {
    Deterministic,
    Strict,
    Explaining,
    Converastion,
    Creative,
}

impl Default for Temperature {
    fn default() -> Self {
        Self::Explaining
    }
}

impl Display for Temperature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Temperature::Deterministic => f.write_str("Deterministic"),
            Temperature::Strict => f.write_str("Strict"),
            Temperature::Explaining => f.write_str("Explaining"),
            Temperature::Converastion => f.write_str("Conversation"),
            Temperature::Creative => f.write_str("Creative"),
        }
    }
}

impl From<Temperature> for f32 {
    fn from(value: Temperature) -> Self {
        match value {
            Temperature::Deterministic => 0f32,
            Temperature::Strict => 0.2f32,
            Temperature::Explaining => 0.65f32,
            Temperature::Converastion => 1.0f32,
            Temperature::Creative => 2f32,
        }
    }
}
