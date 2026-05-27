use crate::services::pipe_processor::pipe_operator::PipeOperator;

pub(crate) struct Lowercase {}

impl Lowercase {
    pub fn new() -> Self {
        Self {}
    }
}

impl PipeOperator for Lowercase {
    fn handle(&self, source: &str) -> anyhow::Result<String> {
        Ok(source.to_lowercase())
    }
}
