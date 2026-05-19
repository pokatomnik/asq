pub(crate) trait LLMProvider {
    fn ask(&self, prompt: impl AsRef<str>) -> anyhow::Result<String>;
}
