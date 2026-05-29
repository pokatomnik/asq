use crate::services::pipe_processor::pipe_operator::PipeOperator;

pub(crate) struct Password;

impl Password {
    pub fn new() -> Self {
        Self {}
    }
}

impl PipeOperator for Password {
    fn handle(&self, prompt: &str) -> anyhow::Result<String> {
        let result = dialoguer::Password::new()
            .with_prompt(prompt)
            .report(false)
            .allow_empty_password(false)
            .report(false)
            .interact()?;

        Ok(result)
    }
}
