use crate::services::pipe_processor::pipe_operator::PipeOperator;

pub(crate) struct Editor;

impl Editor {
    pub fn new() -> Self {
        Self {}
    }
}

impl PipeOperator for Editor {
    fn handle(&self, prompt: &str) -> anyhow::Result<String> {
        let result = dialoguer::Editor::new().edit(prompt)?;
        Ok(result.unwrap_or_default())
    }
}
