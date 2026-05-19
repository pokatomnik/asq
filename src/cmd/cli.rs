use crate::cmd::commands::Commands;
use clap::Parser;

#[derive(Parser)]
#[command(name = "asq")]
#[command(about = "Quick LLM asker")]
#[command(version)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}
