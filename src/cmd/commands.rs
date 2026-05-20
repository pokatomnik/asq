use clap::Subcommand;

use crate::{cmd::providers::ProvidersActions, controllers::onboard::OnboardController};

#[derive(Subcommand)]
pub(crate) enum Commands {
    Onboard(OnboardController),

    #[command(subcommand)]
    Models(ProvidersActions),
}
