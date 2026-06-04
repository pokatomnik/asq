use std::sync::Arc;

use clap::Args;

use crate::controllers::controller::Controller;
use crate::services::parser::Parser;
use crate::services::pipe_processor_presets::user_pipe_processor;
use crate::services::template_env::TemplateEnv;
use crate::utils::describe::Describe;

#[derive(Args, Clone, Debug)]
pub(crate) struct OperatorsListController;

impl Controller for OperatorsListController {
    fn handle(&self) -> anyhow::Result<()> {
        let template_env = Arc::new(TemplateEnv::new(std::env::current_dir()?));
        let pipe_processor = user_pipe_processor(template_env)?;
        let parser = Parser::create(pipe_processor);
        println!("{}", parser.describe());
        Ok(())
    }
}
