use std::{path::PathBuf, sync::Arc};

use crate::services::{
    parser::Parser, pipe_processor::pipe_operator::PipeOperator, template_env::TemplateEnv,
};

pub(crate) struct File {
    template_env: Arc<TemplateEnv>,
}

impl File {
    pub fn new(template_env: Arc<TemplateEnv>) -> Self {
        Self { template_env }
    }
}

impl PipeOperator for File {
    fn handle(&self, path: &str) -> anyhow::Result<String> {
        let path = PathBuf::from(path);

        if path.is_absolute() {
            let result = std::fs::read_to_string(path)?;
            return Ok(result);
        }

        let prompt_dir = self
            .template_env
            .prompt_dir()
            .ok_or_else(|| anyhow::Error::msg("No prompt dir"))?
            .to_path_buf();
        let required_path = prompt_dir.join(path);
        let template_contents = std::fs::read_to_string(required_path.as_path())?;

        let ends_with_md = required_path
            .extension()
            .map(PathBuf::from)
            .map(|v| v.to_string_lossy().to_string())
            .map(|v| v == "md")
            .unwrap_or_default();

        if !ends_with_md {
            return Ok(template_contents);
        }

        let template_env = Arc::new(TemplateEnv::new(required_path));
        let parser = Parser::try_create(template_env)?;
        let prompt_str = parser.compile(template_contents)?;

        Ok(prompt_str)
    }
}
