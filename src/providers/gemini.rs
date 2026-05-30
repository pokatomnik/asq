use crate::entities::proxy::LLMProxy;
use crate::providers::openai_like_provider::OpenAILikeProvider;
use crate::providers::openai_like_provider::ask_model;
use crate::providers::openai_like_provider::ask_name;
use crate::providers::openai_like_provider::ask_token_key;
use crate::utils::init_interactive::InitInteractive;

static API_URL: &'static str = "https://generativelanguage.googleapis.com/v1beta/openai";

pub(crate) fn init_gemini() -> anyhow::Result<OpenAILikeProvider> {
    let name = ask_name("Specify Gemini provider name")?;
    let token_key = ask_token_key(true)?;
    let token = token_key.as_ref().and_then(|tk| std::env::var(tk).ok());
    if token.is_none() {
        anyhow::bail!("Cannot obtain token");
    }

    let proxy_scheme = Option::<LLMProxy>::init_interactive()?;
    let model = ask_model(API_URL, token.as_deref(), proxy_scheme.clone())?;
    let provider = OpenAILikeProvider::new(name, API_URL, token_key, model, proxy_scheme);

    Ok(provider)
}
