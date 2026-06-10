use clap::Parser;

use crate::cmd::commands::Commands::{self, Onboard};
use crate::cmd::memories::MemoriesActions;
use crate::cmd::providers::ProvidersActions;
use crate::cmd::skills::SkillsActions;
use crate::cmd::{cli, operators};
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
                operators::OperatorsActions::List(operators_list_controller) => {
                    operators_list_controller.handle()
                }
            },
            Commands::Memories(memories_actions) => match memories_actions {
                MemoriesActions::List(memories_list_controller) => {
                    memories_list_controller.handle()
                }
                MemoriesActions::Add(memories_add_controller) => memories_add_controller.handle(),
                MemoriesActions::Delete(memories_delete_controller) => {
                    memories_delete_controller.handle()
                }
            },
            Commands::Skills(skills_actions) => match skills_actions {
                SkillsActions::Install(skills_install_controller) => {
                    skills_install_controller.handle()
                }
                SkillsActions::List(skills_list_controller) => skills_list_controller.handle(),
                SkillsActions::Delete(skills_delete_controller) => {
                    skills_delete_controller.handle()
                }
            },
            Commands::Completions(completions_controller) => completions_controller.handle(),
        },
        None => cli.index.handle(),
    };

    result
}
