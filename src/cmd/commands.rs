use clap::Subcommand;

use crate::{
    cmd::{operators::OperatorsActions, providers::ProvidersActions},
    controllers::onboard::OnboardController,
};

#[derive(Subcommand)]
pub(crate) enum Commands {
    Onboard(OnboardController),

    #[command(subcommand)]
    Providers(ProvidersActions),

    #[command(subcommand)]
    Operators(OperatorsActions),
}
