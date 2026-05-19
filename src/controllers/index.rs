use std::path::PathBuf;

use clap::Args;

use crate::controllers::controller::Controller;
use crate::controllers::onboard::OnboardController;
use crate::services::config::Config;
use crate::utils::file_picker::FilePicker;

#[derive(Args, Debug)]
pub(crate) struct IndexController;

impl IndexController {
    pub fn new() -> Self {
        Self {}
    }

    fn handle_select_prompt_file(config: Option<Config>) -> anyhow::Result<()> {
        let config = match config {
            Some(config) => Some(config),
            None => Config::try_read().ok(),
        }
        .ok_or_else(|| anyhow::Error::msg("Failed to read config"))?;

        let root: PathBuf = config.prompts_dir().parse()?;
        let file = dialoguer::FuzzySelect::new()
            .pick_file(&root, |v| v.to_string_lossy().to_string().ends_with(".md"))?;
        println!(
            "{}",
            file.unwrap_or_else(|| PathBuf::default())
                .to_string_lossy()
                .to_string()
        );
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
