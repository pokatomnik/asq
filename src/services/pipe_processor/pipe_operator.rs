pub(crate) trait PipeOperator {
    fn handle(&self, source: &str) -> anyhow::Result<String>;
}
