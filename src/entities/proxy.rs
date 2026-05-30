use serde::{Deserialize, Serialize};

use crate::utils::init_interactive::InitInteractive;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct LLMProxy {
    /// Example:
    /// socks5h://127.0.0.1:1080
    #[serde(rename = "proxyScheme")]
    proxy_scheme: String,
}

impl AsRef<LLMProxy> for LLMProxy {
    fn as_ref(&self) -> &LLMProxy {
        self
    }
}

impl LLMProxy {
    #[allow(unused)]
    fn new(proxy_scheme: impl AsRef<str>) -> Self {
        Self {
            proxy_scheme: proxy_scheme.as_ref().to_string(),
        }
    }

    pub fn proxy_scheme(&self) -> &str {
        &self.proxy_scheme
    }

    fn ask_proxy_scheme() -> anyhow::Result<String> {
        let result = dialoguer::Input::<String>::new()
            .with_prompt("Specify proxy connect URL")
            .default("socks5h://127.0.0.1:1080".to_string())
            .show_default(true)
            .report(false)
            .interact()?;
        Ok(result)
    }
}

impl InitInteractive<Option<LLMProxy>> for Option<LLMProxy> {
    fn init_interactive() -> anyhow::Result<Option<LLMProxy>> {
        let confirmed = dialoguer::Confirm::new()
            .report(false)
            .with_prompt("Add proxy scheme?")
            .default(false)
            .show_default(true)
            .interact()
            .unwrap_or(false);

        if !confirmed {
            return Ok(None);
        }

        let proxy_scheme = LLMProxy::ask_proxy_scheme()?;

        Ok(Some(LLMProxy { proxy_scheme }))
    }
}
