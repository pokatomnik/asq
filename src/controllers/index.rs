use std::path::{Path, PathBuf};

use clap::Args;

use crate::controllers::controller::Controller;
use crate::controllers::onboard::OnboardController;
use crate::entities::llm_provider::LLMProvider;
use crate::entities::llm_provider_kind::LLMProviderKind;
use crate::services::asker::Asker;
use crate::services::config::Config;
use crate::services::prompt_template::PromptTemplate;
use crate::utils::file_picker::FilePicker;

#[derive(Args, Debug)]
pub(crate) struct IndexController;

impl IndexController {
    pub fn new() -> Self {
        Self {}
    }

    fn ensure_config(config: Option<Config>) -> anyhow::Result<Config> {
        let config = match config {
            Some(config) => Some(config),
            None => Config::try_read().ok(),
        }
        .ok_or_else(|| anyhow::Error::msg("Failed to read config"))?;
        Ok(config)
    }

    fn pick_file(root: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
        let file = dialoguer::FuzzySelect::new()
            .pick_file(&root, |v| v.to_string_lossy().to_string().ends_with(".md"))?;
        file.ok_or_else(|| anyhow::Error::msg("Failed to pick file"))
    }

    fn read_template_contents(template_path: impl AsRef<Path>) -> anyhow::Result<String> {
        let result = std::fs::read_to_string(template_path)?;
        Ok(result)
    }

    fn select_template(config: &Config) -> anyhow::Result<PromptTemplate> {
        let root: PathBuf = config.prompts_dir().parse()?;
        let prompt_template_path = Self::pick_file(&root)?;
        let contents = Self::read_template_contents(&prompt_template_path)?;

        let template = PromptTemplate::try_from(contents.as_str())?;
        Ok(template)
    }

    fn select_provider_kind(config: &Config) -> anyhow::Result<&LLMProviderKind> {
        let providers: Vec<&LLMProviderKind> = config.providers().into_iter().collect();
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

    fn handle_select_prompt_file(config: Option<Config>) -> anyhow::Result<()> {
        let config = Self::ensure_config(config)?;
        let provider = Self::select_provider_kind(&config)?;
        let template = Self::select_template(&config)?;
        let asker = Asker::new(template.iter());
        let answers = asker.ask()?;
        let prompt = template.compile(answers)?;

        let response = match provider {
            LLMProviderKind::Ollama(ollama_provider) => ollama_provider.ask(prompt),
        }?;

        println!("{}", response);
        Ok(())
    }
}

impl Controller for IndexController {
    fn handle(&self) -> anyhow::Result<()> {
        let config = Config::try_read();
        match config {
            Ok(config) => Self::handle_select_prompt_file(Some(config)),
            Err(_) => {
                OnboardController::new().handle()?;
                Self::handle_select_prompt_file(None)
            }
        }
    }
}
