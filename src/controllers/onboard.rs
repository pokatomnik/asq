use clap::Args;

use crate::controllers::controller::Controller;
use crate::services::config::Config;
use crate::utils::init_interactive::InitInteractive;

#[derive(Args, Debug, Clone)]
pub(crate) struct OnboardController;

impl OnboardController {
    pub fn new() -> Self {
        Self
    }

    fn ask_overwrite() -> bool {
        dialoguer::Confirm::new()
            .with_prompt("Are you sure to overwrite config?")
            .default(false)
            .show_default(true)
            .interact()
            .unwrap_or(false)
    }

    fn init_and_save() -> anyhow::Result<()> {
        let config = Config::init_interactive()?;
        let result = config.try_write();

        eprintln!("Config saved");

        result
    }
}

impl Controller for OnboardController {
    fn handle(&self) -> anyhow::Result<()> {
        match Config::try_read().is_ok() {
            true => match Self::ask_overwrite() {
                true => Self::init_and_save(),
                false => Ok(()),
            },
            false => Self::init_and_save(),
        }
    }
}
