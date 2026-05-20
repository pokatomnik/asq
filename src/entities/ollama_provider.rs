use std::cell::OnceCell;

use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};

use crate::entities::llm_provider::LLMProvider;
use crate::entities::proxy::LLMProxy;
use crate::entities::system_prompt::SYSTEM_PROMPT;
use crate::utils::client_builder_ext::ClientBuilderExt;
use crate::utils::describe::Describe;
use crate::utils::init_interactive::InitInteractive;
use crate::utils::request_builder_ext::RequestBuilderExt;

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

impl LLMProvider for OllamaProvider {
    fn ask(&self, prompt: impl AsRef<str>) -> anyhow::Result<String> {
        let model = self.model();
        let prompt = prompt.as_ref();

        let body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
            "system": SYSTEM_PROMPT,
            // TODO add options here
            // "options": {}
        });
        let body_json_str = serde_json::to_string(&body)?;

        let client = reqwest::blocking::ClientBuilder::new()
            .with_optional_proxy(self.proxy().as_ref())
            .build()?;

        let url = format!("{}/api/generate", self.enpoint_url().trim_matches('/'));

        let token = self.auth_token();
        let request_builder = client
            .request(Method::POST, url)
            .application_json()
            .with_optional_bearer_token(token);

        let response = request_builder.body(body_json_str).send()?;

        if response.status() != StatusCode::OK {
            anyhow::bail!(format!(
                "Ollama responded with status:{}",
                response.status()
            ))
        }

        let result: OllamaGenerateResponse = response.text()?.try_into()?;

        Ok(result.response)
    }
}

impl Describe for OllamaProvider {
    fn describe(&self) -> String {
        let mut result = String::with_capacity(50);

        result.push_str(format!("Name: {}\n", self.name()).as_str());

        result.push_str(format!("ModelID: {}\n", self.model()).as_str());

        result.push_str(format!("Base URL: {}\n", self.enpoint_url()).as_str());

        let auth_token_env_key = self
            .auth_token_env_key
            .clone()
            .unwrap_or_else(|| "Not set".to_string());
        result.push_str(format!("Auth token env key: {auth_token_env_key}\n",).as_str());

        let proxy = self
            .proxy()
            .map(|p| p.proxy_scheme().to_string())
            .unwrap_or_else(|| "Not set".to_string());
        result.push_str(format!("Proxy: {}\n", proxy.as_str()).as_str());

        result
    }
}

#[derive(serde::Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

impl TryFrom<String> for OllamaGenerateResponse {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let result = serde_json::from_str::<OllamaGenerateResponse>(value.as_str())?;
        Ok(result)
    }
}
