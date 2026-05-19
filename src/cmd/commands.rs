use clap::Subcommand;

use crate::controllers::onboard::OnboardController;

#[derive(Subcommand)]
pub(crate) enum Commands {
    Onboard(OnboardController),
}
