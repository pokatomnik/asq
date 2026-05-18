use serde::{Deserialize, Serialize};

use crate::{entities::llm_provider::LLMProvider, utils::fileman};

static CONFIG_FILE_NAME: &'static str = "asq.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Config {
    #[serde(rename = "providers")]
    providers: Vec<LLMProvider>,

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
    pub fn providers(&self) -> impl Iterator<Item = &LLMProvider> {
        self.providers.iter()
    }

    /// Asks questions interactively and forces the user to do initial setup
    pub fn init_interactive() -> anyhow::Result<Self> {
        println!(
            "You have no configuration yet. Please answer questions to initialize configuration"
        );
        let home_dir = fileman::FileMan::home_dir()
            .ok_or_else(|| anyhow::Error::msg("Can't get home directory"))?;
        let prompts_path = dialoguer::Input::new()
            .with_prompt(
                "Specify full path to your prompts. If path does not exist, asq will create It",
            )
            .default(home_dir.join("prompts"))
            .show_default(true).;
    }
}
