use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum_macros::EnumDiscriminants;

use crate::entities::ollama_provider::OllamaProvider;

#[derive(Debug, Clone, Serialize, Deserialize, EnumDiscriminants)]
#[serde(tag = "kind")]
#[strum_discriminants(name(LLMProviderKind), vis(pub(crate)), derive(Serialize, Deserialize))]
pub(crate) enum LLMProvider {
    Ollama(OllamaProvider),
}

impl Display for LLMProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMProviderKind::Ollama => f.write_str("Ollama"),
        }
    }
}
