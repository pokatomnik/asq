use crate::entities::proxy::LLMProxy;

pub(crate) enum LLMAnswer {
    /// An answer from the LLM that needs to be output to the terminal.
    Text(String),

    /// A request that is processed using third-party tools, such as
    /// opening a browser window with an LLM chat web application and
    /// sending the prompt there.
    External,
}

pub(crate) enum ModelsResponse {
    Models(Vec<String>),
    IntentionallyNoModels,
}

pub(crate) trait LLMProvider {
    fn ask(&self, prompt: impl AsRef<str>) -> anyhow::Result<LLMAnswer>;

    fn list_models(
        endpoint_url: impl AsRef<str>,
        token: Option<&str>,
        proxy: Option<LLMProxy>,
    ) -> anyhow::Result<ModelsResponse>;
}
