use serde::{Deserialize, Serialize};

use crate::entities::proxy::LLMProxy;
use crate::providers::llm_provider::{LLMAnswer, LLMProvider, ModelsResponse};
use crate::utils::describe::Describe;
use crate::utils::init_interactive::InitInteractive;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct DuckDuckGoProvider;

impl DuckDuckGoProvider {
    pub fn name(&self) -> &'static str {
        "DuckDuckGo (Browser)"
    }

    fn format_url(prompt: impl AsRef<str>) -> String {
        static DUCK_AI_BASE_URL: &'static str = "https://duck.ai/chat?ia=chat&origin=funnel_home_website&t=h_&duckai=1&home=1&prompt=1&chip-select=chat";
        let prompt_encoded = urlencoding::encode(prompt.as_ref()).to_string();
        format!("{}&q={}", DUCK_AI_BASE_URL, prompt_encoded)
    }
}

impl InitInteractive<DuckDuckGoProvider> for DuckDuckGoProvider {
    fn init_interactive() -> anyhow::Result<DuckDuckGoProvider> {
        Ok(DuckDuckGoProvider {})
    }
}

impl LLMProvider for DuckDuckGoProvider {
    fn ask(&self, prompt: impl AsRef<str>) -> anyhow::Result<LLMAnswer> {
        open::that(Self::format_url(prompt))?;
        Ok(LLMAnswer::External)
    }

    fn list_models(
        _: impl AsRef<str>,
        _: Option<&str>,
        _: Option<LLMProxy>,
    ) -> anyhow::Result<ModelsResponse> {
        return Ok(ModelsResponse::IntentionallyNoModels);
    }
}

impl Describe for DuckDuckGoProvider {
    fn describe(&self) -> String {
        "Opens a browser with your prompt".to_string()
    }
}
