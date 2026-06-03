use clap::Subcommand;

use crate::cmd::memories::MemoriesActions;
use crate::cmd::operators::OperatorsActions;
use crate::cmd::providers::ProvidersActions;
use crate::controllers::completions::CompletionsController;
use crate::controllers::onboard::OnboardController;

#[derive(Subcommand)]
pub(crate) enum Commands {
    Onboard(OnboardController),

    #[command(subcommand)]
    Providers(ProvidersActions),

    #[command(subcommand)]
    Operators(OperatorsActions),

    #[command(subcommand)]
    Memories(MemoriesActions),

    Completions(CompletionsController),
}
