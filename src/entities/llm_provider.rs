pub(crate) enum LLMAnswer {
    Text(String),
    External,
}

pub(crate) trait LLMProvider {
    fn ask(&self, prompt: impl AsRef<str>) -> anyhow::Result<LLMAnswer>;
}
