use crate::entities::proxy::LLMProxy;
use crate::providers::base_provider::BaseProvider;
use crate::providers::base_provider::ask_endpoint_url;
use crate::providers::base_provider::ask_model;
use crate::providers::base_provider::ask_name;
use crate::providers::base_provider::ask_token_key;
use crate::utils::init_interactive::InitInteractive;

pub(crate) fn init_openai_like() -> anyhow::Result<BaseProvider> {
    let name = ask_name("Specify OpenAI-like provider name")?;
    let endpoint_url = ask_endpoint_url(
        "Specify OpenAI-like endpoint URL",
        Some("http://127.0.0.1:11434/v1"),
    )?;
    let token_key = ask_token_key(true)?;
    let token = token_key.as_ref().and_then(|tk| std::env::var(tk).ok());

    let proxy_scheme = Option::<LLMProxy>::init_interactive()?;

    let model = ask_model(
        endpoint_url.as_str(),
        token.as_deref(),
        proxy_scheme.clone(),
    )?;

    let provider = BaseProvider::new(name, endpoint_url.as_str(), token_key, model, proxy_scheme);

    Ok(provider)
}
