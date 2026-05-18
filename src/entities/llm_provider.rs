use serde::{Deserialize, Serialize};

use crate::entities::ollama_params::OllamaParams;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub(crate) enum LLMProvider {
    Ollama(OllamaParams),
}
