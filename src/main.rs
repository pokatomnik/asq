use clap::Parser;

use crate::cmd::cli;
use crate::cmd::commands::Commands::Onboard;
use crate::controllers::controller::Controller;
use crate::controllers::index::IndexController;

mod cmd;
mod controllers;
mod entities;
mod services;
mod utils;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    let result = match cli.command {
        Some(command) => match command {
            Onboard(onboard_controller) => onboard_controller.handle(),
        },
        None => IndexController::new().handle(),
    };

    if let Err(ref e) = result {
        eprintln!("Error: \"{}\"", e.to_string());
    }

    result
}
