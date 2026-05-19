use std::cell::OnceCell;

use serde::{Deserialize, Serialize};

use crate::{entities::proxy::LLMProxy, utils::init_interactive::InitInteractive};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct OllamaProvider {
    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "endpointURL")]
    endpoint_url: String,

    #[serde(rename = "authTokenEnvKey")]
    auth_token_env_key: Option<String>,

    #[serde(skip)]
    auth_token: OnceCell<Option<String>>,

    #[serde(rename = "model")]
    model: String,

    #[serde(rename = "proxy")]
    proxy: Option<LLMProxy>,
}

impl OllamaProvider {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn enpoint_url(&self) -> &str {
        &self.endpoint_url
    }

    pub fn auth_token(&self) -> Option<&str> {
        let env_key = self.auth_token_env_key.as_deref()?;
        self.auth_token
            .get_or_init(|| std::env::var(env_key).ok())
            .as_ref()
            .map(|v| v.as_str())
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
            .show_default(true)
            .default("http://127.0.0.1:11434".to_string())
            .interact()?;

        Ok(result)
    }

    fn ask_token_key() -> anyhow::Result<Option<String>> {
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
            .with_prompt("Ask never store secure keys inside plaintext configuration files. We ask you to provide env vaiable to obtain It")
            .report(false)
            .default("OLLAMA_API_KEY".to_string())
            .show_default(true)
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
        let token_key = Self::ask_token_key()?;
        let model = Self::ask_model()?;
        let proxy_scheme = Option::<LLMProxy>::init_interactive()?;
        Ok(Self {
            name,
            endpoint_url,
            auth_token_env_key: token_key,
            auth_token: OnceCell::default(),
            model,
            proxy: proxy_scheme,
        })
    }
}
