use clap::{Args, CommandFactory};
use clap_complete::{Shell, generate};

use crate::{cmd::cli::Cli, controllers::controller::Controller};

#[derive(Args, Clone, Debug)]
pub(crate) struct CompletionsController {
    #[arg(long, short, default_value_t = Shell::Bash, help = "Shell to generate completion script for")]
    shell: Shell,
}

impl Controller for CompletionsController {
    fn handle(&self) -> anyhow::Result<()> {
        let mut cmd = Cli::command();
        let name = &cmd.get_name().to_string();
        generate(self.shell, &mut cmd, name, &mut std::io::stdout());
        Ok(())
    }
}
