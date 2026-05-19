use std::path::{Path, PathBuf};

use clap::Args;

use crate::controllers::controller::Controller;
use crate::controllers::onboard::OnboardController;
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

    fn select_template(config: Option<Config>) -> anyhow::Result<PromptTemplate> {
        let config = Self::ensure_config(config)?;
        let root: PathBuf = config.prompts_dir().parse()?;
        let prompt_template_path = Self::pick_file(&root)?;
        let contents = Self::read_template_contents(&prompt_template_path)?;

        let template = PromptTemplate::try_from(contents.as_str())?;
        Ok(template)
    }

    fn handle_select_prompt_file(config: Option<Config>) -> anyhow::Result<()> {
        let template = Self::select_template(config)?;
        let asker = Asker::new(template.iter());
        let answers = asker.ask()?;
        let prompt = template.compile(answers)?;

        println!("{}", prompt);
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
