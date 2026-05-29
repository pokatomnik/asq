use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::{Args, ValueEnum};

use crate::controllers::controller::Controller;
use crate::controllers::onboard::OnboardController;
use crate::entities::llm_provider_kind::LLMProviderKind;
use crate::entities::message::Message;
use crate::entities::prompt::{FrondmatterKind, Prompt};
use crate::entities::role::Role;
use crate::providers::llm_provider::LLMProvider;
use crate::services::config::Config;
use crate::services::history::History;
use crate::services::parser::Parser;
use crate::services::template_env::TemplateEnv;
use crate::utils::file_picker::FilePicker;
use crate::utils::with_spinner::with_spinner;

#[derive(Clone, Copy, Debug, ValueEnum)]
#[clap(rename_all = "kebab-case")]
enum OutputMode {
    Markdown,
    Plain,
}

impl Display for OutputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputMode::Markdown => f.write_str("markdown"),
            OutputMode::Plain => f.write_str("plain"),
        }
    }
}

#[derive(Args, Debug)]
pub(crate) struct IndexController {
    #[arg(long, short, default_value_t = false, help = "Select LLM provider")]
    select_provider: bool,

    #[arg(
        long = "continue",
        short = 'c',
        default_value_t = false,
        help = "Continue previous dialog"
    )]
    r#continue: bool,

    #[arg(long = "output", short = 'o', default_value_t = OutputMode::Markdown, help = "Terminal output type")]
    output: OutputMode,
}

impl IndexController {
    fn ensure_config(config: Option<Config>) -> anyhow::Result<Config> {
        let config = match config {
            Some(config) => Some(config),
            None => Config::try_read().ok(),
        }
        .ok_or_else(|| anyhow::Error::msg("Failed to read config"))?;
        Ok(config)
    }

    fn pick_file(root: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
        let file = dialoguer::FuzzySelect::new()
            .report(false)
            .clear(true)
            .pick_file(&root, |v| v.to_string_lossy().to_string().ends_with(".md"))?;
        file.ok_or_else(|| anyhow::Error::msg("Failed to pick file"))
    }

    fn read_template_contents(template_path: impl AsRef<Path>) -> anyhow::Result<String> {
        let result = std::fs::read_to_string(template_path)?;
        Ok(result)
    }

    fn select_template(config: &Config) -> anyhow::Result<(PathBuf, String)> {
        let root: PathBuf = config.prompts_dir().parse()?;
        let prompt_template_path = Self::pick_file(&root)?;
        let contents = Self::read_template_contents(&prompt_template_path)?;
        Ok((prompt_template_path, contents))
    }

    fn select_provider_kind(&self, config: &Config) -> anyhow::Result<LLMProviderKind> {
        if let Some(last_used_provider) = config.last_used_provider()
            && !self.select_provider
        {
            return Ok(last_used_provider.clone());
        }
        let providers: Vec<&LLMProviderKind> = config.providers().into_iter().collect();
        let idx = dialoguer::FuzzySelect::new()
            .report(false)
            .clear(true)
            .with_prompt("Select LLM provider")
            .default(0)
            .highlight_matches(true)
            .items(&providers)
            .interact()?;
        let provider = &providers
            .get(idx)
            .ok_or_else(|| anyhow::Error::msg("Failed to select LLM provider"))?;
        let provider = (***provider).clone();

        Ok(provider.to_owned())
    }

    fn prepare_raw_prompt(prompt: &str, raw: &str) -> String {
        let warning_header = include_str!("./raw_frontmatter.md");
        format!("{prompt}\n{warning_header}\n{raw}")
    }

    fn prepare_prompt(prompt: &Prompt) -> impl AsRef<str> {
        match (prompt.frontmatter(), prompt.prompt()) {
            (Some(frontmatter), prompt) => match frontmatter {
                FrondmatterKind::Raw(raw) => Self::prepare_raw_prompt(prompt, raw),
            },
            (None, prompt) => prompt.to_string(),
        }
    }

    fn print_markdown(response: impl AsRef<str>) {
        let output = marcli::render(response.as_ref(), &Default::default());
        println!("{}", output);
    }

    fn print_plain(response: impl AsRef<str>) {
        println!("{}", response.as_ref())
    }

    fn ask_text(prompt: &str) -> anyhow::Result<String> {
        let result = dialoguer::Input::new()
            .report(false)
            .with_prompt(prompt)
            .interact()?;
        Ok(result)
    }

    fn handle_ask_model(&self, config: Option<Config>) -> anyhow::Result<()> {
        let mut config = Self::ensure_config(config)?;
        let history = History::new(self.r#continue);

        let provider = self.select_provider_kind(&config)?;

        let prompt_text = match !history.is_empty() && self.r#continue {
            true => Self::ask_text("Your question")?,
            false => {
                let (prompt_path, contents) = Self::select_template(&config)?;
                let template_env = Arc::new(TemplateEnv::new(&prompt_path));
                let parser = Parser::try_create(template_env)?;
                let prompt_str = parser.compile(&contents)?;
                let prompt = Prompt::new(prompt_str);
                Self::prepare_prompt(&prompt).as_ref().to_string()
            }
        };

        history.with_history(|messages| {
            let answer =
                with_spinner(
                    format!("{} answer:", &provider.to_string()),
                    || match provider {
                        LLMProviderKind::Ollama(ref ollama_provider) => {
                            ollama_provider.ask(&prompt_text, messages.clone())
                        }
                        LLMProviderKind::Openrouter(ref openrouter_provider) => {
                            openrouter_provider.ask(&prompt_text, messages.clone())
                        }
                        LLMProviderKind::Deepseek(ref deepseek_provider) => {
                            deepseek_provider.ask(&prompt_text, messages.clone())
                        }
                        LLMProviderKind::OpenAILike(ref openai_like_provider) => {
                            openai_like_provider.ask(&prompt_text, messages.clone())
                        }
                    },
                )?;

            let mut messages = messages;
            messages.push(Message::new(Role::User, &prompt_text));
            messages.push(Message::new(Role::Assistant, answer.response()));

            config.set_last_used_provider(Some(provider));
            config.try_write()?;

            match self.output {
                OutputMode::Markdown => Self::print_markdown(&answer.response()),
                OutputMode::Plain => Self::print_plain(&answer.response()),
            }

            Ok(messages)
        })?;

        Ok(())
    }
}

impl Controller for IndexController {
    fn handle(&self) -> anyhow::Result<()> {
        let config = Config::try_read();
        match config {
            Ok(config) => self.handle_ask_model(Some(config)),
            Err(_) => {
                OnboardController::new().handle()?;
                self.handle_ask_model(None)
            }
        }
    }
}
