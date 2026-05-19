use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum_macros::EnumDiscriminants;

use crate::entities::ollama_provider::OllamaProvider;

#[derive(Debug, Clone, Serialize, Deserialize, EnumDiscriminants)]
#[serde(tag = "kind")]
#[strum_discriminants(
    name(LLMProviderKindOnly),
    vis(pub(crate)),
    derive(Serialize, Deserialize)
)]
pub(crate) enum LLMProviderKind {
    Ollama(OllamaProvider),
}

impl Display for LLMProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMProviderKind::Ollama(ollama_provider) => f.write_str(ollama_provider.name()),
        }
    }
}

impl Display for LLMProviderKindOnly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMProviderKindOnly::Ollama => f.write_str("Ollama"),
        }
    }
}
