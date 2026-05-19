use serde::{Deserialize, Serialize};

use crate::{entities::proxy::LLMProxy, utils::init_interactive::InitInteractive};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct OllamaProvider {
    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "endpointURL")]
    endpoint_url: String,

    #[serde(rename = "authToken")]
    auth_token: Option<String>,

    #[serde(rename = "model")]
    model: String,

    #[serde(rename = "proxy")]
    proxy: Option<LLMProxy>,
}

impl OllamaProvider {
    pub fn enpoint_url(&self) -> &str {
        &self.endpoint_url
    }

    pub fn auth_token(&self) -> Option<&str> {
        self.auth_token.as_deref()
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn proxy(&self) -> Option<&LLMProxy> {
        self.proxy.as_ref()
    }

    fn ask_name() -> anyhow::Result<String> {
        let result = dialoguer::Input::new()
            .with_prompt("Specify LLM provider name, example: \"Ollama|gemma4:e4b\"")
            .interact()?;
        Ok(result)
    }

    fn ask_endpoint_url() -> anyhow::Result<String> {
        let result = dialoguer::Input::new()
            .with_prompt("Specify Ollama endpoint URL")
            .interact()?;

        Ok(result)
    }

    fn ask_token() -> anyhow::Result<Option<String>> {
        let confirmed = dialoguer::Confirm::new()
            .with_prompt("Specify token?")
            .default(false)
            .show_default(true)
            .interact()
            .unwrap_or(false);
        if !confirmed {
            return Ok(None);
        }

        let token = dialoguer::Input::<String>::new()
            .with_prompt("Specify token")
            .interact()?;

        return Ok(Some(token));
    }

    fn ask_model() -> anyhow::Result<String> {
        let model = dialoguer::Input::<String>::new()
            .with_prompt("Specify model")
            .interact()?;
        Ok(model)
    }
}

impl InitInteractive<OllamaProvider> for OllamaProvider {
    fn init_interactive() -> anyhow::Result<OllamaProvider> {
        let name = Self::ask_name()?;
        let endpoint_url = Self::ask_endpoint_url()?;
        let token = Self::ask_token()?;
        let model = Self::ask_model()?;
        let proxy_scheme = Option::<LLMProxy>::init_interactive()?;
        Ok(Self {
            name,
            endpoint_url,
            auth_token: token,
            model,
            proxy: proxy_scheme,
        })
    }
}
