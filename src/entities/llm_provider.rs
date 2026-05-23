pub(crate) enum LLMAnswer {
    /// An answer from the LLM that needs to be output to the terminal.
    Text(String),

    /// A request that is processed using third-party tools, such as
    /// opening a browser window with an LLM chat web application and
    /// sending the prompt there.
    External,
}

pub(crate) trait LLMProvider {
    fn ask(&self, prompt: impl AsRef<str>) -> anyhow::Result<LLMAnswer>;
}
