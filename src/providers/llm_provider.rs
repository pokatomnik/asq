use serde::{Deserialize, Serialize};

use crate::entities::{message::Message, proxy::LLMProxy};

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

pub(crate) enum ModelsResponse {
    Models(Vec<String>),

    /// There are currently no providers without a public model list, but we keep this invariant for flexibility.
    #[allow(unused)]
    IntentionallyNoModels,
}

pub(crate) trait LLMProvider {
    fn ask(
        &self,
        prompt: impl AsRef<str>,
        messages: impl IntoIterator<Item = Message>,
    ) -> anyhow::Result<LLMAnswer>;

    fn list_models(
        endpoint_url: impl AsRef<str>,
        token: Option<&str>,
        proxy: Option<LLMProxy>,
    ) -> anyhow::Result<ModelsResponse>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum LLMProviderMessageRole {
    #[serde(rename = "system")]
    System,

    #[serde(rename = "assistant")]
    Assistant,

    #[serde(rename = "user")]
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LLMProviderMessage {
    #[serde(rename = "role")]
    role: LLMProviderMessageRole,

    #[serde(rename = "content")]
    content: String,
}

impl LLMProviderMessage {
    pub fn new(role: LLMProviderMessageRole, content: &str) -> Self {
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
