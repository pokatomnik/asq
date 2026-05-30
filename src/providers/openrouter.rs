use crate::{
    entities::proxy::LLMProxy,
    providers::base_provider::{BaseProvider, ask_model, ask_name, ask_token_key},
    utils::init_interactive::InitInteractive,
};

static API_URL: &'static str = "https://openrouter.ai/api/v1";

pub(crate) fn init_openrouter() -> anyhow::Result<BaseProvider> {
    let name = ask_name("Specify Openrouter provider name")?;
    let token_key = ask_token_key(true)?;
    let token = token_key.as_ref().and_then(|tk| std::env::var(tk).ok());
    if token.is_none() {
        anyhow::bail!("Cannot obtain token");
    }
    let proxy_scheme = Option::<LLMProxy>::init_interactive()?;
    let model = ask_model(API_URL, token.as_deref(), proxy_scheme.clone())?;

    let provider = BaseProvider::new(name, API_URL, token_key, model, proxy_scheme);

    Ok(provider)
}
