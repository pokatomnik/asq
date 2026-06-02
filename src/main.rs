use clap::Parser;

use crate::cmd::cli;
use crate::cmd::commands::Commands::{self, Onboard};
use crate::cmd::providers::ProvidersActions;
use crate::controllers::controller::Controller;

mod cmd;
mod controllers;
mod entities;
mod providers;
mod services;
mod tools;
mod utils;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    let result = match cli.command {
        Some(command) => match command {
            Onboard(onboard_controller) => onboard_controller.handle(),
            Commands::Providers(models_actions) => match models_actions {
                ProvidersActions::List(models_list_controller) => models_list_controller.handle(),
                ProvidersActions::Add(models_add_controller) => models_add_controller.handle(),
                ProvidersActions::Delete(models_delete_controller) => {
                    models_delete_controller.handle()
                }
            },
            Commands::Operators(operators_actions) => match operators_actions {
                cmd::operators::OperatorsActions::List(operators_list_controller) => {
                    operators_list_controller.handle()
                }
            },
        },
        None => cli.index.handle(),
    };

    result
}
