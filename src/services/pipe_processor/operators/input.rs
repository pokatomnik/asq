use crate::services::pipe_processor::pipe_operator::PipeOperator;

pub(crate) struct Input;

impl Input {
    pub fn new() -> Self {
        Self {}
    }
}

impl PipeOperator for Input {
    fn handle(&self, prompt: &str) -> anyhow::Result<String> {
        let result = dialoguer::Input::new()
            .with_prompt(prompt)
            .report(false)
            .allow_empty(false)
            .interact()?;

        Ok(result)
    }
}
