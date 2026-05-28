use std::cell::OnceCell;
use std::fmt::Display;

use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};

use crate::entities::consts::DEFAULT_TIMEOUT;
use crate::entities::message::Message;
use crate::entities::proxy::LLMProxy;
use crate::entities::role::Role;
use crate::providers::llm_provider::{
    LLMAnswer, LLMProvider, LLMProviderMessage, LLMProviderMessageRole, LLMProviderRequestBody,
    ModelsResponse,
};
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
            .report(false)
            .with_prompt("Specify LLM provider name, example: \"Ollama|gemma4:e4b\"")
            .interact()?;
        Ok(result)
    }

    fn ask_endpoint_url() -> anyhow::Result<String> {
        let result = dialoguer::Input::new()
            .report(false)
            .with_prompt("Specify Ollama endpoint URL")
            .show_default(true)
            .default("http://127.0.0.1:11434".to_string())
            .interact()?;

        Ok(result)
    }

    fn ask_token_key() -> anyhow::Result<Option<String>> {
        let confirmed = dialoguer::Confirm::new()
            .report(false)
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

    fn ask_model(
        endpoint_url: impl AsRef<str>,
        token: Option<&str>,
        proxy: Option<LLMProxy>,
    ) -> anyhow::Result<String> {
        let models = Self::list_models(endpoint_url, token, proxy)?;
        let ModelsResponse::Models(models) = models else {
            anyhow::bail!("Cannot select model")
        };
        let model_idx = dialoguer::FuzzySelect::new()
            .report(false)
            .clear(true)
            .with_prompt("Specify model")
            .items(&models)
            .default(0)
            .highlight_matches(true)
            .interact()?;
        let Some(model) = &models.get(model_idx) else {
            anyhow::bail!("Cannot select model");
        };
        Ok(model.to_owned().to_owned())
    }
}

impl InitInteractive<OllamaProvider> for OllamaProvider {
    fn init_interactive() -> anyhow::Result<OllamaProvider> {
        let name = Self::ask_name()?;
        let endpoint_url = Self::ask_endpoint_url()?;
        let token_key = Self::ask_token_key()?;
        let token = token_key.as_ref().and_then(|tk| std::env::var(tk).ok());

        let proxy_scheme = Option::<LLMProxy>::init_interactive()?;
        let ask_model_url = format!("{}/v1/models", endpoint_url);
        let model = Self::ask_model(ask_model_url, token.as_deref(), proxy_scheme.clone())?;
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
    fn ask(
        &self,
        prompt: impl AsRef<str>,
        history: impl IntoIterator<Item = Message>,
    ) -> anyhow::Result<LLMAnswer> {
        let model = self.model();
        let prompt = prompt.as_ref();

        let mut messages = history
            .into_iter()
            .map(|m| {
                let role = match m.role() {
                    Role::System => LLMProviderMessageRole::System,
                    Role::User => LLMProviderMessageRole::User,
                    Role::Assistant => LLMProviderMessageRole::Assistant,
                };
                LLMProviderMessage::new(role, m.contents())
            })
            .collect::<Vec<LLMProviderMessage>>();

        messages.push(LLMProviderMessage::new(
            LLMProviderMessageRole::User,
            prompt,
        ));

        let body = LLMProviderRequestBody::new(model, messages, false);

        let body_json_str = serde_json::to_string(&body)?;

        let client = reqwest::blocking::ClientBuilder::new()
            .timeout(DEFAULT_TIMEOUT)
            .with_optional_proxy(self.proxy().as_ref())
            .build()?;

        let url = format!(
            "{}/v1/chat/completions",
            self.enpoint_url().trim_matches('/')
        );

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
        let response = result
            .choices
            .get(0)
            .ok_or_else(|| anyhow::Error::msg("No response from LLM"))?;

        Ok(LLMAnswer::new(response.message.content.as_str()))
    }

    fn list_models(
        endpoint_url: impl AsRef<str>,
        token: Option<&str>,
        proxy: Option<LLMProxy>,
    ) -> anyhow::Result<ModelsResponse> {
        let client = reqwest::blocking::ClientBuilder::new()
            .timeout(DEFAULT_TIMEOUT)
            .with_optional_proxy(proxy)
            .build()?;

        let response = client
            .request(Method::GET, endpoint_url.as_ref())
            .application_json()
            .with_optional_bearer_token::<&str>(token)
            .send()?;

        let result_json = response.text()?;

        let parsed_result = serde_json::from_str::<OllamaModelsResponse>(result_json.as_str())?;
        let model_names = parsed_result
            .data
            .iter()
            .map(|v| v.id.to_owned())
            .collect::<Vec<String>>();

        Ok(ModelsResponse::Models(model_names))
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
    #[serde(rename = "model")]
    #[allow(unused)]
    model: String,

    #[serde(rename = "choices")]
    choices: Vec<OllamaGenerateResponseChoice>,
}

#[derive(serde::Deserialize)]
struct OllamaGenerateResponseChoice {
    #[serde(rename = "index")]
    #[allow(unused)]
    index: usize,

    #[serde(rename = "finish_reason")]
    #[allow(unused)]
    finish_reason: OllamaGenerateResponseFinishReason,

    #[serde(rename = "message")]
    message: OllamaGenerateResponseMessage,
}

#[derive(serde::Deserialize)]
struct OllamaGenerateResponseMessage {
    #[serde(rename = "role")]
    #[allow(unused)]
    role: OllamaGenerateResponseRole,

    #[serde(rename = "content")]
    content: String,
}

#[derive(serde::Deserialize, Clone, Debug, Copy, PartialEq, Eq)]
enum OllamaGenerateResponseRole {
    #[serde(rename = "system")]
    System,

    #[serde(rename = "assistant")]
    Assistant,

    #[serde(rename = "user")]
    User,
}

impl Display for OllamaGenerateResponseRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OllamaGenerateResponseRole::System => f.write_str("system"),
            OllamaGenerateResponseRole::Assistant => f.write_str("assistant"),
            OllamaGenerateResponseRole::User => f.write_str("user"),
        }
    }
}

#[derive(serde::Deserialize, Clone, Copy, PartialEq, PartialOrd)]
enum OllamaGenerateResponseFinishReason {
    #[serde(rename = "stop")]
    Stop,

    #[serde(rename = "length")]
    Length,

    #[serde(rename = "tool_calls")]
    ToolCalls,

    #[serde(rename = "content_filter")]
    ContentFilter,

    #[serde(rename = "error")]
    Error,
}

impl Display for OllamaGenerateResponseFinishReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OllamaGenerateResponseFinishReason::Stop => f.write_str("stop"),
            OllamaGenerateResponseFinishReason::Length => f.write_str("length"),
            OllamaGenerateResponseFinishReason::ToolCalls => f.write_str("tool_calls"),
            OllamaGenerateResponseFinishReason::ContentFilter => f.write_str("content_filter"),
            OllamaGenerateResponseFinishReason::Error => f.write_str("error"),
        }
    }
}

impl TryFrom<String> for OllamaGenerateResponse {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let result = serde_json::from_str::<OllamaGenerateResponse>(value.as_str())?;
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaModelsResponse {
    data: Vec<ModelDescription>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ModelDescription {
    id: String,
}
