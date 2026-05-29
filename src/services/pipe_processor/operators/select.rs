use crate::services::pipe_processor::pipe_operator::PipeOperator;

pub(crate) struct Select;

impl Select {
    pub fn new() -> Self {
        Self {}
    }
}

impl PipeOperator for Select {
    fn handle(&self, source: &str) -> anyhow::Result<String> {
        let lines = source
            .split("\n")
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        if lines.len() == 0 {
            anyhow::bail!("Nothing to select")
        }
        let answer = dialoguer::FuzzySelect::new()
            .items(&lines)
            .default(0)
            .highlight_matches(true)
            .clear(true)
            .report(false)
            .default(0)
            .interact()?;
        let selected_line = lines
            .get(answer)
            .ok_or_else(|| anyhow::Error::msg("Cannot select"))?;

        Ok(selected_line.to_string())
    }
}
