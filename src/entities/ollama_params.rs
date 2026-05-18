use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct OllamaParams {
    #[serde(rename = "endpointURL")]
    endpoint_url: String,

    #[serde(rename = "authToken")]
    auth_token: String,

    #[serde(rename = "model")]
    model: String,
}

impl OllamaParams {
    pub fn new(endpoint_url: String, auth_token: String, model: String) -> Self {
        Self {
            endpoint_url,
            auth_token,
            model,
        }
    }

    pub fn enpoint_url(&self) -> &str {
        &self.endpoint_url
    }

    pub fn auth_token(&self) -> &str {
        &self.auth_token
    }

    pub fn model(&self) -> &str {
        &self.model
    }
}
