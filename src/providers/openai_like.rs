use std::{cell::OnceCell, fmt::Display};

use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};

use crate::entities::consts::DEFAULT_TIMEOUT;
use crate::entities::proxy::LLMProxy;
use crate::entities::system_prompt::SYSTEM_PROMPT;
use crate::providers::llm_provider::LLMAnswer;
use crate::providers::llm_provider::LLMProvider;
use crate::providers::llm_provider::ModelsResponse;
use crate::utils::client_builder_ext::ClientBuilderExt;
use crate::utils::describe::Describe;
use crate::utils::init_interactive::InitInteractive;
use crate::utils::request_builder_ext::RequestBuilderExt;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct OpenAILikeProvider {
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

impl OpenAILikeProvider {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn endpoint_url(&self) -> &str {
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
            .with_prompt("Specify LLM provider name, example: \"OpenAI-like|gpt-oss:20b\"")
            .interact()?;
        Ok(result)
    }

    fn ask_endpoint_url() -> anyhow::Result<String> {
        let result = dialoguer::Input::new()
            .report(false)
            .with_prompt("Specify OpenAI like endpoint URL")
            .show_default(true)
            .default("http://127.0.0.1:1234".to_string())
            .interact()?;

        Ok(result)
    }

    fn ask_token_key() -> anyhow::Result<Option<String>> {
        let token = dialoguer::Input::<String>::new()
            .with_prompt("Ask never store secure keys inside plaintext configuration files. We ask you to provide env vaiable to obtain It")
            .report(false)
            .show_default(true)
            .interact()?;

        return Ok(Some(token));
    }

    fn ask_model(
        models_url: impl AsRef<str>,
        token: Option<&str>,
        proxy: Option<LLMProxy>,
    ) -> anyhow::Result<String> {
        let models = Self::list_models(models_url.as_ref(), token, proxy)?;
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

impl InitInteractive<OpenAILikeProvider> for OpenAILikeProvider {
    fn init_interactive() -> anyhow::Result<OpenAILikeProvider> {
        let name = Self::ask_name()?;
        let endpoint_url = Self::ask_endpoint_url()?;
        let token_key = Self::ask_token_key()?;
        let token = token_key.as_ref().and_then(|tk| std::env::var(tk).ok());

        let proxy_scheme = Option::<LLMProxy>::init_interactive()?;
        let ask_model_url = format!("{}/models", endpoint_url.trim().trim_matches('/'));
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

impl LLMProvider for OpenAILikeProvider {
    fn ask(&self, prompt: impl AsRef<str>) -> anyhow::Result<LLMAnswer> {
        let model = self.model();
        let prompt = prompt.as_ref();

        let body = serde_json::json!({
            "model": model,
            "messages": [{
                "role": "system",
                "content": SYSTEM_PROMPT
            }, {
                "role": "user",
                "content": prompt
            }],
            "stream": false,
        });
        let body_json_str = serde_json::to_string(&body)?;

        let client = reqwest::blocking::ClientBuilder::new()
            .timeout(DEFAULT_TIMEOUT)
            .with_optional_proxy(self.proxy().as_ref())
            .build()?;

        let token = self.auth_token();
        let url = format!(
            "{}/chat/completions",
            self.endpoint_url.trim().trim_matches('/')
        );
        let request_builder = client
            .request(Method::POST, url)
            .application_json()
            .with_optional_bearer_token(token);

        let response = request_builder.body(body_json_str).send()?;

        if response.status() != StatusCode::OK {
            anyhow::bail!(format!(
                "OpenAI like server responded with status: {}",
                response.status()
            ))
        }

        let result: OpenAILikeGenerateResponse = response.text()?.try_into()?;

        let llm_response_message = result
            .choices
            .get(0)
            .ok_or_else(|| anyhow::Error::msg("No response from model"))?;

        if llm_response_message.finish_reason != OpenAILikeGenerateResponseFinishReason::Stop {
            anyhow::bail!(
                "Unexpected LLM response: {}",
                llm_response_message.finish_reason
            );
        }

        if llm_response_message.message.role != OpenAILikeGenerateResponseRole::Assistant {
            anyhow::bail!(
                "Unexpected LLM response role: {}",
                llm_response_message.message.role
            );
        }

        Ok(LLMAnswer::Text(
            llm_response_message.message.content.to_owned(),
        ))
    }

    fn list_models(
        models_url: impl AsRef<str>,
        token: Option<&str>,
        proxy: Option<LLMProxy>,
    ) -> anyhow::Result<ModelsResponse> {
        let client = reqwest::blocking::ClientBuilder::new()
            .timeout(DEFAULT_TIMEOUT)
            .with_optional_proxy(proxy)
            .build()?;

        let response = client
            .request(Method::GET, models_url.as_ref())
            .application_json()
            .with_optional_bearer_token(token)
            .send()?;

        let result_json = response.text()?;

        let parsed_result = serde_json::from_str::<OpenAILikeModelsResponse>(result_json.as_str())?;
        let model_names = parsed_result
            .data
            .iter()
            .map(|v| v.id.to_owned())
            .collect::<Vec<String>>();

        Ok(ModelsResponse::Models(model_names))
    }
}

#[derive(serde::Deserialize)]
struct OpenAILikeGenerateResponse {
    #[serde(rename = "model")]
    #[allow(unused)]
    model: String,

    #[serde(rename = "choices")]
    choices: Vec<OpenAILikeGenerateResponseChoice>,
}

#[derive(serde::Deserialize)]
struct OpenAILikeGenerateResponseChoice {
    #[serde(rename = "index")]
    #[allow(unused)]
    index: usize,

    #[serde(rename = "finish_reason")]
    finish_reason: OpenAILikeGenerateResponseFinishReason,

    #[serde(rename = "message")]
    message: OpenAILikeGenerateResponseMessage,
}

#[derive(serde::Deserialize)]
struct OpenAILikeGenerateResponseMessage {
    #[serde(rename = "role")]
    role: OpenAILikeGenerateResponseRole,

    #[serde(rename = "content")]
    content: String,
}

#[derive(serde::Deserialize, Clone, Debug, Copy, PartialEq, Eq)]
enum OpenAILikeGenerateResponseRole {
    #[serde(rename = "system")]
    System,

    #[serde(rename = "assistant")]
    Assistant,

    #[serde(rename = "user")]
    User,
}

impl Display for OpenAILikeGenerateResponseRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenAILikeGenerateResponseRole::System => f.write_str("system"),
            OpenAILikeGenerateResponseRole::Assistant => f.write_str("assistant"),
            OpenAILikeGenerateResponseRole::User => f.write_str("user"),
        }
    }
}

#[derive(serde::Deserialize, Clone, Copy, PartialEq, PartialOrd)]
enum OpenAILikeGenerateResponseFinishReason {
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

impl Display for OpenAILikeGenerateResponseFinishReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenAILikeGenerateResponseFinishReason::Stop => f.write_str("stop"),
            OpenAILikeGenerateResponseFinishReason::Length => f.write_str("length"),
            OpenAILikeGenerateResponseFinishReason::ToolCalls => f.write_str("tool_calls"),
            OpenAILikeGenerateResponseFinishReason::ContentFilter => f.write_str("content_filter"),
            OpenAILikeGenerateResponseFinishReason::Error => f.write_str("error"),
        }
    }
}

impl Describe for OpenAILikeProvider {
    fn describe(&self) -> String {
        let mut result = String::with_capacity(50);
        result.push_str(format!("Name: {}\n", self.name()).as_str());
        result.push_str(format!("ModelID: {}\n", self.model()).as_str());
        result.push_str(format!("Base URL: {}\n", self.endpoint_url()).as_str());
        let auth_token_env_key = self
            .auth_token_env_key
            .clone()
            .unwrap_or("Not set".to_string());
        result.push_str(format!("Auth token env key: {auth_token_env_key}\n",).as_str());
        let proxy = self
            .proxy()
            .map(|p| p.proxy_scheme().to_string())
            .unwrap_or_else(|| "Not set".to_string());
        result.push_str(format!("Proxy: {}\n", proxy.as_str()).as_str());

        result
    }
}

impl TryFrom<String> for OpenAILikeGenerateResponse {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let result = serde_json::from_str::<OpenAILikeGenerateResponse>(value.as_str())?;
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAILikeModelsResponse {
    data: Vec<ModelDescription>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ModelDescription {
    id: String,
}
