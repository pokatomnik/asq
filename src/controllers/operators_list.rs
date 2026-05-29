use std::sync::Arc;

use clap::Args;

use crate::{
    controllers::controller::Controller,
    services::{parser::Parser, template_env::TemplateEnv},
    utils::describe::Describe,
};

#[derive(Args, Clone, Debug)]
pub(crate) struct OperatorsListController;

impl Controller for OperatorsListController {
    fn handle(&self) -> anyhow::Result<()> {
        let parser = Parser::try_create(Arc::new(TemplateEnv::new(std::env::current_dir()?)))?;
        println!("{}", parser.describe());
        Ok(())
    }
}
