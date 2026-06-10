use crate::{cmd::commands::Commands, controllers::index::IndexController};
use clap::Parser;

#[derive(Parser)]
#[command(name = "asq")]
#[command(about = "Quick LLM asker")]
#[command(version)]
pub(crate) struct Cli {
    #[command(flatten)]
    pub index: IndexController,

    #[command(subcommand)]
    pub command: Option<Commands>,
}
