use clap::Args;

use crate::{
    controllers::controller::Controller, entities::llm_provider_kind::LLMProviderKind,
    services::config::Config,
};

#[derive(Args, Clone, Debug)]
pub(crate) struct ProvidersDeleteController;

impl ProvidersDeleteController {
    fn select_provider_idx(config: &Config) -> anyhow::Result<usize> {
        let providers: Vec<&LLMProviderKind> = config.providers().into_iter().collect();
        if providers.is_empty() {
            anyhow::bail!("You have no providers configured. Add a new one.")
        }
        let idx = dialoguer::FuzzySelect::new()
            .report(false)
            .clear(true)
            .with_prompt("Select LLM provider")
            .default(0)
            .highlight_matches(true)
            .items(&providers)
            .interact()?;

        Ok(idx)
    }
}

impl Controller for ProvidersDeleteController {
    fn handle(&self) -> anyhow::Result<()> {
        let mut config = Config::try_read()?;
        let provider_idx = Self::select_provider_idx(&config)?;
        config.delete_provider_by_idx(provider_idx);

        config.try_write()?;

        Ok(())
    }
}
