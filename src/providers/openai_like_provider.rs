use std::sync::{Arc, OnceLock};
use std::{cell::OnceCell, fmt::Display};

use reqwest::blocking::Response;
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::entities::consts::DEFAULT_TIMEOUT;
use crate::entities::message::Message;
use crate::entities::proxy::LLMProxy;
use crate::entities::role::Role;
use crate::entities::temperature::Temperature;
use crate::entities::tool_call::ToolCall;
use crate::tools::tools_registry::ToolsRegistry;
use crate::utils::client_builder_ext::ClientBuilderExt;
use crate::utils::describe::Describe;
use crate::utils::request_builder_ext::RequestBuilderExt;
use crate::utils::with_spinner::with_spinner;

#[derive(Clone, Serialize, Deserialize)]
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

    #[serde(rename = "temperature")]
    temperature: Option<Temperature>,

    #[serde(skip)]
    tools_registry: OnceLock<Arc<ToolsRegistry>>,
}

impl OpenAILikeProvider {
    pub fn new(
        name: impl Into<String>,
        endpoint_url: impl Into<String>,
        auth_token_env_key: Option<impl Into<String>>,
        model: impl Into<String>,
        proxy: Option<LLMProxy>,
        temperature: Option<Temperature>,
    ) -> Self {
        Self {
            name: name.into(),
            endpoint_url: endpoint_url.into(),
            auth_token_env_key: auth_token_env_key.map(Into::into),
            model: model.into(),
            proxy,
            auth_token: Default::default(),
            temperature: temperature,
            tools_registry: Default::default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn endpoint_url(&self) -> &str {
        &self.endpoint_url
    }

    pub fn tools_registry<'a>(&'a self) -> &'a ToolsRegistry {
        self.tools_registry
            .get_or_init(|| Arc::new(ToolsRegistry::new()))
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

    pub fn temperature(&self) -> Temperature {
        self.temperature.unwrap_or_default()
    }

    pub fn proxy(&self) -> Option<&LLMProxy> {
        self.proxy.as_ref()
    }

    fn process_tools(&self, tool_calls: &[ToolCall]) -> Vec<Message> {
        let mut results = Vec::with_capacity(tool_calls.len());
        let tools = self.tools_registry();

        for tool_call in tool_calls.iter() {
            let Some(function) = tool_call.function() else {
                continue;
            };

            let (tool_call_result, tool_call_errors) =
                tools.call_tool(function.name(), function.arguments());
            let mut res = Vec::new();
            res.extend(tool_call_result.iter());
            res.extend(tool_call_errors.iter());
            let as_str = res
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join("\n");
            let message =
                Message::new(Role::Tool, &as_str).with_tool_call_id(tool_call.id().to_string());
            results.push(message);
        }

        results
    }

    fn send_chat_completion(
        &self,
        history: &[Message],
    ) -> anyhow::Result<OpenAILikeProviderGenerateResponse> {
        let temperature: f32 = self.temperature().into();
        let messages = history
            .iter()
            .map(LLMProviderMessage::from)
            .collect::<Vec<LLMProviderMessage>>();

        let body = LLMProviderRequestBody::new(
            self.model(),
            messages,
            self.tools_registry().tool_definitions()?,
            temperature,
            false,
        );

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

        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            return self.send_chat_completion(history);
        }

        if response.status() != StatusCode::OK {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            anyhow::bail!(format!(
                "Server responded with status: {status}, text: {error_text}"
            ))
        }

        let result: OpenAILikeProviderGenerateResponse = response.text()?.try_into()?;

        Ok(result)
    }
}

impl LLMProvider for OpenAILikeProvider {
    fn ask(&self, prompt: impl AsRef<str>, mut history: Vec<Message>) -> anyhow::Result<LLMAnswer> {
        const MAX_TOOL_ITERATIONS: usize = 8;

        history.push(Message::new(Role::User, prompt.as_ref()));

        for _ in 0..MAX_TOOL_ITERATIONS {
            let result = self.send_chat_completion(&history)?;
            let llm_response_choice = result
                .choices
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::Error::msg("No response from model"))?;

            let llm_response_message = llm_response_choice.message;
            if llm_response_message.role != Role::Assistant {
                anyhow::bail!(
                    "Unexpected LLM response role: {}",
                    llm_response_message.role
                );
            }

            match llm_response_choice.finish_reason {
                OpenAILikeProviderGenerateResponseFinishReason::Stop => {
                    let llm_response_str = llm_response_message.content.unwrap_or_default();
                    history.push(Message::new(Role::Assistant, llm_response_str.as_str()));
                    return Ok(LLMAnswer::new(llm_response_str.as_str(), history));
                }
                OpenAILikeProviderGenerateResponseFinishReason::ToolCalls => {
                    let tool_calls = llm_response_message.tool_calls.unwrap_or_default();
                    if tool_calls.is_empty() {
                        anyhow::bail!("LLM requested tool calls but returned no tool calls");
                    }

                    history.push(Message::assistant_tool_call(
                        llm_response_message.content,
                        tool_calls.clone(),
                    ));
                    history.extend(self.process_tools(&tool_calls));
                }
                unexpected => anyhow::bail!("Unexpected LLM response: {unexpected}"),
            }
        }

        anyhow::bail!("Too many consecutive tool calls from LLM")
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
        "Loading models done",
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

    let parsed_result =
        serde_json::from_str::<OpenAILikeProviderModelsResponse>(result_json.as_str())?;
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

pub(crate) fn ask_temperature() -> Option<Temperature> {
    let confirmed = dialoguer::Confirm::new()
        .with_prompt("Specify temperature?")
        .default(false)
        .show_default(true)
        .report(false)
        .interact()
        .unwrap_or_default();

    if !confirmed {
        return None;
    }

    let options = vec![
        Temperature::Deterministic,
        Temperature::Strict,
        Temperature::Explaining,
        Temperature::Converastion,
        Temperature::Creative,
    ];

    let idx = dialoguer::FuzzySelect::new()
        .with_prompt("Specify response temperature")
        .items(&options)
        .default(0)
        .highlight_matches(true)
        .report(false)
        .clear(true)
        .interact()
        .unwrap_or(0);
    let selected_temperature = options.get(idx).map(ToOwned::to_owned).unwrap_or_default();

    Some(selected_temperature)
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
struct OpenAILikeProviderGenerateResponse {
    #[serde(rename = "choices")]
    choices: Vec<OpenAILikeProviderGenerateResponseChoice>,
}

#[derive(Deserialize)]
struct OpenAILikeProviderGenerateResponseChoice {
    #[serde(rename = "finish_reason")]
    finish_reason: OpenAILikeProviderGenerateResponseFinishReason,

    #[serde(rename = "message")]
    message: OpenAILikeProviderGenerateResponseMessage,
}

#[derive(serde::Deserialize)]
struct OpenAILikeProviderGenerateResponseMessage {
    #[serde(rename = "role")]
    role: Role,

    #[serde(rename = "content")]
    content: Option<String>,

    #[serde(rename = "tool_calls")]
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(serde::Deserialize, Clone, Copy, PartialEq, PartialOrd)]
enum OpenAILikeProviderGenerateResponseFinishReason {
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

impl Display for OpenAILikeProviderGenerateResponseFinishReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenAILikeProviderGenerateResponseFinishReason::Stop => f.write_str("stop"),
            OpenAILikeProviderGenerateResponseFinishReason::Length => f.write_str("length"),
            OpenAILikeProviderGenerateResponseFinishReason::ToolCalls => f.write_str("tool_calls"),
            OpenAILikeProviderGenerateResponseFinishReason::ContentFilter => {
                f.write_str("content_filter")
            }
            OpenAILikeProviderGenerateResponseFinishReason::Error => f.write_str("error"),
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
        result.push_str(format!("Auth token env key: {auth_token_env_key}\n").as_str());
        result.push_str(format!("Model temperature: {}\n", self.temperature()).as_str());
        let proxy = self
            .proxy()
            .map(|p| p.proxy_scheme().to_string())
            .unwrap_or_else(|| "Not set".to_string());
        result.push_str(format!("Proxy: {}\n", proxy.as_str()).as_str());

        result
    }
}

impl TryFrom<String> for OpenAILikeProviderGenerateResponse {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let result = serde_json::from_str::<OpenAILikeProviderGenerateResponse>(value.as_str())?;
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAILikeProviderModelsResponse {
    data: Vec<OpenAILikeProviderModelDescription>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OpenAILikeProviderModelDescription {
    id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LLMProviderMessage {
    #[serde(rename = "role")]
    role: Role,

    #[serde(rename = "content", skip_serializing_if = "Option::is_none")]
    content: Option<String>,

    #[serde(rename = "tool_call_id", skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,

    #[serde(rename = "tool_calls", skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
}

impl From<&Message> for LLMProviderMessage {
    fn from(value: &Message) -> Self {
        Self {
            role: *value.role(),
            content: value.content().map(ToString::to_string),
            tool_call_id: value.tool_call_id().map(ToString::to_string),
            tool_calls: value.tool_calls().map(ToOwned::to_owned),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
enum ToolChoice {
    #[serde(rename = "auto")]
    Auto,
}

impl Display for ToolChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolChoice::Auto => f.write_str("auto"),
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

    #[serde(rename = "temperature")]
    temperature: f32,

    #[serde(rename = "tools")]
    tools: Vec<Value>,

    #[serde(rename = "tool_choice")]
    tool_choice: ToolChoice,
}

impl LLMProviderRequestBody {
    pub fn new(
        model: &str,
        messages: impl IntoIterator<Item = LLMProviderMessage>,
        tools: Vec<Value>,
        temperature: f32,
        stream: bool,
    ) -> Self {
        Self {
            model: model.to_string(),
            messages: messages.into_iter().collect(),
            tools: tools,
            tool_choice: ToolChoice::Auto,
            stream,
            temperature,
        }
    }
}

pub(crate) struct LLMAnswer {
    response: String,
    messages: Vec<Message>,
}

impl LLMAnswer {
    pub fn new(response: &str, messages: Vec<Message>) -> Self {
        Self {
            response: response.to_string(),
            messages,
        }
    }

    pub fn response(&self) -> &str {
        &self.response
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }
}

pub(crate) trait LLMProvider {
    fn ask(&self, prompt: impl AsRef<str>, messages: Vec<Message>) -> anyhow::Result<LLMAnswer>;
}
