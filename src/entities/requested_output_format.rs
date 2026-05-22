use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// This enum serves two pusposes:
/// - Tell the model how to format output
/// - Tell asq cli how to display this output
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub(crate) enum RequestedOutputFormat {
    #[serde(rename = "json")]
    JSON,

    #[serde(rename = "yaml")]
    YAML,

    #[serde(rename = "plain")]
    Plain,

    #[serde(rename = "markdown")]
    Markdown,
}

impl Display for RequestedOutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestedOutputFormat::JSON => f.write_str("JSON"),
            RequestedOutputFormat::YAML => f.write_str("YAML"),
            RequestedOutputFormat::Plain => f.write_str("Plain Text"),
            RequestedOutputFormat::Markdown => f.write_str("Markdown"),
        }
    }
}
