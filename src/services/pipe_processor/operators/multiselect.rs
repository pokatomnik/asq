use crate::services::pipe_processor::pipe_operator::PipeOperator;

pub(crate) struct Multiselect;

impl Multiselect {
    pub fn new() -> Self {
        Self {}
    }
}

impl PipeOperator for Multiselect {
    fn handle(&self, source: &str) -> anyhow::Result<String> {
        let lines = source
            .split("\n")
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        let answer = dialoguer::MultiSelect::new()
            .items(&lines)
            .clear(true)
            .report(false)
            .interact()?;

        let results = answer
            .iter()
            .filter_map(|idx| lines.get(*idx))
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        Ok(results.join("\n"))
    }
}
