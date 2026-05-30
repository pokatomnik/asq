use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub(crate) enum Role {
    #[serde(rename = "user")]
    User,

    #[serde(rename = "assistant")]
    Assistant,

    #[serde(rename = "system")]
    System,

    #[serde(rename = "tools")]
    Tools,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::User => f.write_str("user"),
            Role::Assistant => f.write_str("assistant"),
            Role::System => f.write_str("system"),
            Role::Tools => f.write_str("tools"),
        }
    }
}
