use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum_macros::EnumDiscriminants;

use crate::{providers::openai_like_provider::OpenAILikeProvider, utils::describe::Describe};

#[derive(Debug, Clone, Serialize, Deserialize, EnumDiscriminants)]
#[serde(tag = "kind")]
#[strum_discriminants(
    name(LLMProviderKindOnly),
    vis(pub(crate)),
    derive(Serialize, Deserialize)
)]
pub(crate) enum LLMProviderKind {
    Openrouter(OpenAILikeProvider),
    Deepseek(OpenAILikeProvider),
    OpenAILike(OpenAILikeProvider),
    Gemini(OpenAILikeProvider),
}

impl AsRef<LLMProviderKind> for LLMProviderKind {
    fn as_ref(&self) -> &LLMProviderKind {
        self
    }
}

impl Describe for LLMProviderKind {
    fn describe(&self) -> String {
        match self {
            LLMProviderKind::Openrouter(openrouter_provider) => openrouter_provider.describe(),
            LLMProviderKind::Deepseek(deepseek_provider) => deepseek_provider.describe(),
            LLMProviderKind::OpenAILike(openai_like_provider) => openai_like_provider.describe(),
            LLMProviderKind::Gemini(gemini_provider) => gemini_provider.describe(),
        }
    }
}

impl Display for LLMProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMProviderKind::Openrouter(openrouter_provider) => {
                f.write_str(openrouter_provider.name())
            }
            LLMProviderKind::Deepseek(deepseek_provider) => f.write_str(deepseek_provider.name()),
            LLMProviderKind::OpenAILike(openai_like_provider) => {
                f.write_str(openai_like_provider.name())
            }
            LLMProviderKind::Gemini(gemini) => f.write_str(gemini.name()),
        }
    }
}

impl Display for LLMProviderKindOnly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMProviderKindOnly::Openrouter => f.write_str("Openrouter"),
            LLMProviderKindOnly::Deepseek => f.write_str("Deepseek"),
            LLMProviderKindOnly::OpenAILike => f.write_str("OpenAI-like"),
            LLMProviderKindOnly::Gemini => f.write_str("Gemini"),
        }
    }
}
