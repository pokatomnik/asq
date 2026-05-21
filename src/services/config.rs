use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::entities::duckduckgo_provider::DuckDuckGoProvider;
use crate::entities::llm_provider_kind::{LLMProviderKind, LLMProviderKindOnly};
use crate::entities::ollama_provider::OllamaProvider;
use crate::entities::openrouter_provider::OpenrouterProvider;
use crate::utils::fileman;
use crate::utils::init_interactive::InitInteractive;

static CONFIG_FILE_NAME: &'static str = "asq.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Config {
    #[serde(rename = "providers")]
    providers: Vec<LLMProviderKind>,

    /// Full path to prompts
    #[serde(rename = "promptsDir")]
    prompts_dir: String,
}

impl Config {
    pub fn try_read() -> anyhow::Result<Self> {
        let config_bytes = fileman::FileMan::read_data(CONFIG_FILE_NAME)?;
        let config = serde_json::from_slice::<Self>(config_bytes.as_ref())?;
        Ok(config)
    }

    pub fn try_write(&self) -> anyhow::Result<()> {
        let serialized = serde_json::to_string_pretty(self)?;
        fileman::FileMan::save_data(CONFIG_FILE_NAME, serialized)?;
        Ok(())
    }

    /// Get full path to prompts dir
    pub fn prompts_dir(&self) -> &str {
        &self.prompts_dir
    }

    /// Get all providers list
    pub fn providers(&self) -> impl Iterator<Item = &LLMProviderKind> {
        self.providers.iter()
    }

    fn ask_prompt_paths() -> anyhow::Result<PathBuf> {
        let home_dir = fileman::FileMan::home_dir()
            .ok_or_else(|| anyhow::Error::msg("Can't get home directory"))?;
        let result = dialoguer::Input::new()
            .with_prompt(
                "Specify full path to your prompts. If path does not exist, asq will create It",
            )
            .report(false)
            .default(home_dir.join("prompts").to_string_lossy().to_string())
            .show_default(true)
            .interact()
            .map(PathBuf::from)?;

        Ok(result)
    }

    pub fn add_provider(&mut self, provider: LLMProviderKind) {
        self.providers.push(provider);
    }

    pub fn delete_provider_by_idx(&mut self, idx: usize) {
        self.providers = self
            .providers
            .clone()
            .iter()
            .enumerate()
            .filter_map(|(current_idx, provider)| match current_idx == idx {
                true => None,
                false => Some(provider.to_owned()),
            })
            .collect();
    }

    fn ask_providers() -> anyhow::Result<Vec<LLMProviderKind>> {
        let mut providers = Vec::new();

        let mut proceed = true;

        while proceed {
            let Ok(new_provider) = Self::ask_provider() else {
                anyhow::bail!("Incorrect provider")
            };
            providers.push(new_provider);

            proceed = dialoguer::Confirm::new()
                .with_prompt("Add one more model?")
                .default(false)
                .show_default(true)
                .interact()
                .unwrap_or(false)
        }

        Ok(providers)
    }

    fn ask_provider() -> anyhow::Result<LLMProviderKind> {
        let kind = Self::ask_kind()?;
        match kind {
            LLMProviderKindOnly::Ollama => {
                Ok(LLMProviderKind::Ollama(OllamaProvider::init_interactive()?))
            }
            LLMProviderKindOnly::Openrouter => Ok(LLMProviderKind::Openrouter(
                OpenrouterProvider::init_interactive()?,
            )),
            LLMProviderKindOnly::DuckDuckGo => Ok(LLMProviderKind::DuckDuckGo(
                DuckDuckGoProvider::init_interactive()?,
            )),
        }
    }

    fn ask_kind() -> anyhow::Result<LLMProviderKindOnly> {
        let all_kinds = vec![
            LLMProviderKindOnly::Ollama,
            LLMProviderKindOnly::Openrouter,
            LLMProviderKindOnly::DuckDuckGo,
        ];
        let kind_idx = dialoguer::FuzzySelect::new()
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

impl InitInteractive<Config> for Config {
    /// Asks questions interactively and forces the user to do initial setup
    fn init_interactive() -> anyhow::Result<Self> {
        println!(
            "You have no configuration yet. Please answer questions to initialize configuration"
        );
        let prompts_path = Self::ask_prompt_paths()?;
        let providers = Self::ask_providers()?;

        let result = Self {
            prompts_dir: prompts_path.to_string_lossy().to_string(),
            providers,
        };
        Ok(result)
    }
}
