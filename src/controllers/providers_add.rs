use clap::Args;

use crate::controllers::controller::Controller;
use crate::entities::llm_provider_kind::{LLMProviderKind, LLMProviderKindOnly};
use crate::providers::deepseek::init_deepseek;
use crate::providers::openai_like::init_openai_like;
use crate::providers::openrouter::init_openrouter;
use crate::services::config::Config;

#[derive(Args, Clone, Debug)]
pub(crate) struct ProvidersAddController {}

impl ProvidersAddController {
    fn ask_kind() -> anyhow::Result<LLMProviderKindOnly> {
        let all_kinds = vec![
            LLMProviderKindOnly::Openrouter,
            LLMProviderKindOnly::Deepseek,
            LLMProviderKindOnly::OpenAILike,
        ];
        let kind_idx = dialoguer::FuzzySelect::new()
            .report(false)
            .clear(true)
            .with_prompt("Select LLM provider kind")
            .default(0)
            .highlight_matches(true)
            .items(&all_kinds)
            .interact()?;
        let kind = &all_kinds.get(kind_idx);
        kind.cloned()
            .ok_or_else(|| anyhow::Error::msg("Provider kind not found"))
    }
}

impl Controller for ProvidersAddController {
    fn handle(&self) -> anyhow::Result<()> {
        let new_provider_kind = Self::ask_kind()?;
        let new_provider = match new_provider_kind {
            LLMProviderKindOnly::Openrouter => LLMProviderKind::Openrouter(init_openrouter()?),
            LLMProviderKindOnly::Deepseek => LLMProviderKind::Deepseek(init_deepseek()?),
            LLMProviderKindOnly::OpenAILike => LLMProviderKind::OpenAILike(init_openai_like()?),
        };
        let mut config = Config::try_read()?;

        config.add_provider(new_provider);

        config.try_write()
    }
}
