use std::{cell::OnceCell, fmt::Display};

use reqwest::blocking::Response;
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};

use crate::entities::consts::DEFAULT_TIMEOUT;
use crate::entities::message::Message;
use crate::entities::proxy::LLMProxy;
use crate::entities::role::Role;
use crate::utils::client_builder_ext::ClientBuilderExt;
use crate::utils::describe::Describe;
use crate::utils::request_builder_ext::RequestBuilderExt;
use crate::utils::with_spinner::with_spinner;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct BaseProvider {
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

impl BaseProvider {
    pub fn new(
        name: impl Into<String>,
        endpoint_url: impl Into<String>,
        auth_token_env_key: Option<impl Into<String>>,
        model: impl Into<String>,
        proxy: Option<LLMProxy>,
    ) -> Self {
        Self {
            name: name.into(),
            endpoint_url: endpoint_url.into(),
            auth_token_env_key: auth_token_env_key.map(Into::into),
            model: model.into(),
            proxy,
            auth_token: Default::default(),
        }
    }

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
}

impl LLMProvider for BaseProvider {
    fn ask(
        &self,
        prompt: impl AsRef<str>,
        history: impl IntoIterator<Item = Message>,
    ) -> anyhow::Result<LLMAnswer> {
        let model = self.model();
        let prompt = prompt.as_ref();

        let mut messages = history
            .into_iter()
            .map(|m| LLMProviderMessage::new(*m.role(), m.contents()))
            .collect::<Vec<LLMProviderMessage>>();

        messages.push(LLMProviderMessage::new(Role::User, prompt));

        let body = LLMProviderRequestBody::new(model, messages, false);

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
                "Server responded with status: {}",
                response.status()
            ))
        }

        let result: BaseProviderGenerateResponse = response.text()?.try_into()?;

        let llm_response_message = result
            .choices
            .get(0)
            .ok_or_else(|| anyhow::Error::msg("No response from model"))?;

        if llm_response_message.finish_reason != BaseProviderGenerateResponseFinishReason::Stop {
            anyhow::bail!(
                "Unexpected LLM response: {}",
                llm_response_message.finish_reason
            );
        }

        if llm_response_message.message.role != Role::Assistant {
            anyhow::bail!(
                "Unexpected LLM response role: {}",
                llm_response_message.message.role
            );
        }

        Ok(LLMAnswer::new(
            llm_response_message.message.content.as_str(),
        ))
    }
}

pub(crate) fn list_models(
    endpoint_url: impl AsRef<str>,
    token: Option<&str>,
    proxy: Option<LLMProxy>,
) -> anyhow::Result<Vec<String>> {
    let client = reqwest::blocking::ClientBuilder::new()
        .timeout(DEFAULT_TIMEOUT)
        .with_optional_proxy(proxy)
        .build()?;

    let endpoint_url = format!("{}/models", endpoint_url.as_ref().trim().trim_matches('/'));
    let response = with_spinner(
        "Loading models...",
        || -> anyhow::Result<Response, anyhow::Error> {
            let result = client
                .request(Method::GET, endpoint_url.as_str())
                .application_json()
                .with_optional_bearer_token(token)
                .send()?;
            Ok(result)
        },
    )?;

    let result_json = response.text()?;

    let parsed_result = serde_json::from_str::<BaseProviderModelsResponse>(result_json.as_str())?;
    let model_names = parsed_result
        .data
        .iter()
        .map(|v| v.id.to_owned())
        .collect::<Vec<String>>();

    Ok(model_names)
}

pub(crate) fn ask_name(prompt: &str) -> anyhow::Result<String> {
    let result = dialoguer::Input::new()
        .report(false)
        .with_prompt(prompt)
        .interact()?;
    Ok(result)
}

pub(crate) fn ask_endpoint_url(prompt: &str, default: Option<&str>) -> anyhow::Result<String> {
    let mut result = dialoguer::Input::<String>::new()
        .report(false)
        .with_prompt(prompt);
    if let Some(default) = default {
        result = result.default(default.into()).show_default(true);
    }

    Ok(result.interact()?)
}

pub(crate) fn ask_token_key(required: bool) -> anyhow::Result<Option<String>> {
    let mut confirm = true;
    if !required {
        confirm = dialoguer::Confirm::new()
            .report(false)
            .default(false)
            .show_default(true)
            .with_prompt("Specify token?")
            .interact()
            .unwrap_or_default();
    }
    if !confirm {
        return Ok(None);
    }
    let token = dialoguer::Input::<String>::new()
        .with_prompt("Ask never store secure keys inside plaintext configuration files. We ask you to provide env vaiable to obtain It")
        .report(false)
        .show_default(true)
        .interact()?;

    Ok(Some(token))
}

pub(crate) fn ask_model(
    endpoint_url: impl AsRef<str>,
    token: Option<&str>,
    proxy: Option<LLMProxy>,
) -> anyhow::Result<String> {
    let models = list_models(endpoint_url.as_ref(), token, proxy)?;

    if models.len() == 0 {
        anyhow::bail!("No models there");
    }

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

#[derive(Deserialize)]
struct BaseProviderGenerateResponse {
    #[serde(rename = "choices")]
    choices: Vec<BaseGenerateResponseChoice>,
}

#[derive(Deserialize)]
struct BaseGenerateResponseChoice {
    #[serde(rename = "finish_reason")]
    finish_reason: BaseProviderGenerateResponseFinishReason,

    #[serde(rename = "message")]
    message: BaseGenerateResponseMessage,
}

#[derive(serde::Deserialize)]
struct BaseGenerateResponseMessage {
    #[serde(rename = "role")]
    role: Role,

    #[serde(rename = "content")]
    content: String,
}

#[derive(serde::Deserialize, Clone, Copy, PartialEq, PartialOrd)]
enum BaseProviderGenerateResponseFinishReason {
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

impl Display for BaseProviderGenerateResponseFinishReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseProviderGenerateResponseFinishReason::Stop => f.write_str("stop"),
            BaseProviderGenerateResponseFinishReason::Length => f.write_str("length"),
            BaseProviderGenerateResponseFinishReason::ToolCalls => f.write_str("tool_calls"),
            BaseProviderGenerateResponseFinishReason::ContentFilter => {
                f.write_str("content_filter")
            }
            BaseProviderGenerateResponseFinishReason::Error => f.write_str("error"),
        }
    }
}

impl Describe for BaseProvider {
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

impl TryFrom<String> for BaseProviderGenerateResponse {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let result = serde_json::from_str::<BaseProviderGenerateResponse>(value.as_str())?;
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct BaseProviderModelsResponse {
    data: Vec<BaseProviderModelDescription>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BaseProviderModelDescription {
    id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LLMProviderMessage {
    #[serde(rename = "role")]
    role: Role,

    #[serde(rename = "content")]
    content: String,
}

impl LLMProviderMessage {
    pub fn new(role: Role, content: &str) -> Self {
        Self {
            role,
            content: content.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LLMProviderRequestBody {
    #[serde(rename = "model")]
    model: String,

    #[serde(rename = "messages")]
    messages: Vec<LLMProviderMessage>,

    #[serde(rename = "stream")]
    stream: bool,
}

impl LLMProviderRequestBody {
    pub fn new(
        model: &str,
        messages: impl IntoIterator<Item = LLMProviderMessage>,
        stream: bool,
    ) -> Self {
        Self {
            model: model.to_string(),
            messages: messages.into_iter().collect(),
            stream,
        }
    }
}

pub(crate) struct LLMAnswer {
    response: String,
}

impl LLMAnswer {
    pub fn new(response: &str) -> Self {
        Self {
            response: response.to_string(),
        }
    }

    pub fn response(&self) -> &str {
        &self.response
    }
}

pub(crate) trait LLMProvider {
    fn ask(
        &self,
        prompt: impl AsRef<str>,
        messages: impl IntoIterator<Item = Message>,
    ) -> anyhow::Result<LLMAnswer>;
}
