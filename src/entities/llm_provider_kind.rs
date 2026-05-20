use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum_macros::EnumDiscriminants;

use crate::entities::ollama_provider::OllamaProvider;
use crate::entities::openrouter_provider::OpenrouterProvider;
use crate::utils::describe::Describe;

#[derive(Debug, Clone, Serialize, Deserialize, EnumDiscriminants)]
#[serde(tag = "kind")]
#[strum_discriminants(
    name(LLMProviderKindOnly),
    vis(pub(crate)),
    derive(Serialize, Deserialize)
)]
pub(crate) enum LLMProviderKind {
    Ollama(OllamaProvider),
    Openrouter(OpenrouterProvider),
}

impl AsRef<LLMProviderKind> for LLMProviderKind {
    fn as_ref(&self) -> &LLMProviderKind {
        self
    }
}

impl Describe for LLMProviderKind {
    fn describe(&self) -> String {
        match self {
            LLMProviderKind::Ollama(ollama_provider) => ollama_provider.describe(),
            LLMProviderKind::Openrouter(openrouter_provider) => openrouter_provider.describe(),
        }
    }
}

impl Display for LLMProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMProviderKind::Ollama(ollama_provider) => f.write_str(ollama_provider.name()),
            LLMProviderKind::Openrouter(openrouter_provider) => {
                f.write_str(openrouter_provider.name())
            }
        }
    }
}

impl Display for LLMProviderKindOnly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMProviderKindOnly::Ollama => f.write_str("Ollama"),
            LLMProviderKindOnly::Openrouter => f.write_str("Openrouter"),
        }
    }
}
