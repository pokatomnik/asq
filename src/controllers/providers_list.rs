use crate::controllers::controller::Controller;
use crate::entities::llm_provider_kind::LLMProviderKind;
use crate::services::config::Config;
use crate::utils::describe::Describe;
use clap::Args;

#[derive(Clone, Debug, Args)]
pub struct ProvidersListController;

impl ProvidersListController {
    fn select_provider_kind(config: &Config) -> anyhow::Result<&LLMProviderKind> {
        let providers: Vec<&LLMProviderKind> = config.providers().into_iter().collect();
        if providers.is_empty() {
            anyhow::bail!("You have no providers configured. Add a new one.")
        }
        let idx = dialoguer::FuzzySelect::new()
            .with_prompt("Select LLM provider")
            .default(0)
            .highlight_matches(true)
            .items(&providers)
            .interact()?;
        let provider = &providers
            .get(idx)
            .ok_or_else(|| anyhow::Error::msg("Failed to select LLM provider"))?;

        Ok(provider)
    }
}

impl Controller for ProvidersListController {
    fn handle(&self) -> anyhow::Result<()> {
        let Ok(config) = Config::try_read() else {
            eprintln!("You haven't set things up. Try \"onboard\" first");
            return Ok(());
        };

        let selected = Self::select_provider_kind(&config)?;

        let description = selected.describe();

        println!("{}", description);

        Ok(())
    }
}
