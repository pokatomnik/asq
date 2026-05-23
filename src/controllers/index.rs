use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bat::PrettyPrinter;
use clap::Args;
use termimad::MadSkin;
use yaml_serde::Value;

use crate::controllers::controller::Controller;
use crate::controllers::onboard::OnboardController;
use crate::entities::llm_provider::{LLMAnswer, LLMProvider};
use crate::entities::llm_provider_kind::LLMProviderKind;
use crate::entities::placeholder::Placeholder;
use crate::entities::prompt::FrondmatterKind::{self, Typed};
use crate::entities::prompt::{Prompt, TypedFrontmatter};
use crate::entities::requested_output_format::RequestedOutputFormat;
use crate::services::asker::Asker;
use crate::services::config::Config;
use crate::services::prompt_template::PromptTemplate;
use crate::utils::file_picker::FilePicker;

#[derive(Args, Debug)]
pub(crate) struct IndexController;

impl IndexController {
    pub fn new() -> Self {
        Self {}
    }

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

    fn select_template(config: &Config) -> anyhow::Result<PromptTemplate> {
        let root: PathBuf = config.prompts_dir().parse()?;
        let prompt_template_path = Self::pick_file(&root)?;
        let contents = Self::read_template_contents(&prompt_template_path)?;

        let template = PromptTemplate::try_from(contents.as_str())?;
        Ok(template)
    }

    fn select_provider_kind(config: &Config) -> anyhow::Result<&LLMProviderKind> {
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

        Ok(provider)
    }

    fn prepare_typed_prompt(
        prompt: &str,
        frontmatter: &TypedFrontmatter,
    ) -> anyhow::Result<String> {
        let template = PromptTemplate::try_from(include_str!("./typed_frontmatter.md"))?;
        let answers = {
            let mut answers = HashMap::new();
            answers.insert(
                Placeholder::new("prompt", false, Option::<String>::None),
                prompt.to_string(),
            );
            answers.insert(
                Placeholder::new("format", false, Option::<String>::None),
                frontmatter.output().to_string(),
            );
            answers
        };
        let compiled = template.compile(answers)?;
        Ok(compiled.prompt().to_string())
    }

    fn prepare_untyped_prompt(prompt: &str, frontmatter: &Value) -> anyhow::Result<String> {
        let json = serde_json::to_string_pretty(frontmatter)?;
        let template = PromptTemplate::try_from(include_str!("./untyped_frontmatter.md"))?;
        let answers = {
            let mut answers = HashMap::new();
            answers.insert(
                Placeholder::new("prompt", false, Option::<String>::None),
                prompt.to_string(),
            );
            answers.insert(
                Placeholder::new("value", false, Option::<String>::None),
                json,
            );
            answers
        };
        let compiled = template.compile(answers)?;
        Ok(compiled.prompt().to_string())
    }

    fn prepare_raw_prompt(prompt: &str, raw: &str) -> anyhow::Result<String> {
        let template = PromptTemplate::try_from(include_str!("./raw_frontmatter.md"))?;
        let answers = {
            let mut answers = HashMap::new();
            answers.insert(
                Placeholder::new("prompt", false, Option::<String>::None),
                prompt.to_string(),
            );
            answers.insert(
                Placeholder::new("raw", false, Option::<String>::None),
                raw.to_string(),
            );
            answers
        };
        let compiled = template.compile(answers)?;

        Ok(compiled.prompt().to_string())
    }

    fn prepare_prompt(prompt: &Prompt) -> impl AsRef<str> {
        match (prompt.frontmatter(), prompt.prompt()) {
            (Some(frontmatter), prompt) => match frontmatter {
                FrondmatterKind::Typed(typed_frontmatter) => {
                    Self::prepare_typed_prompt(prompt, typed_frontmatter)
                        .unwrap_or_else(move |_| prompt.to_string())
                }
                FrondmatterKind::Untyped(untyped_frontmatter) => {
                    Self::prepare_untyped_prompt(prompt, untyped_frontmatter)
                        .unwrap_or_else(move |_| prompt.to_string())
                }
                FrondmatterKind::Raw(raw) => Self::prepare_raw_prompt(prompt, raw)
                    .unwrap_or_else(move |_| prompt.to_string()),
            },
            (None, prompt) => prompt.to_string(),
        }
    }

    fn print_plain(response: impl AsRef<str>) {
        println!("{}", response.as_ref())
    }

    fn print_with_highlight(response: impl AsRef<str>, format: RequestedOutputFormat) {
        let format = match format {
            RequestedOutputFormat::JSON => "json",
            RequestedOutputFormat::YAML => "yaml",
            RequestedOutputFormat::Plain => "plain",
            RequestedOutputFormat::Markdown => "markdown",
        };
        let result = PrettyPrinter::new()
            .input_from_bytes(response.as_ref().as_bytes())
            .language(format)
            .colored_output(true)
            .line_numbers(false)
            .header(false)
            .grid(false)
            .rule(false)
            .vcs_modification_markers(false)
            .snip(false)
            .print();
        if let Err(e) = result {
            eprintln!("Error: {}", e.to_string())
        }
    }

    fn print_markdown(response: impl AsRef<str>) {
        let skin = MadSkin::default();
        println!("{}", skin.term_text(response.as_ref()));
    }

    fn handle_prompt_response(prompt: &Prompt, response: &LLMAnswer) {
        match (prompt.frontmatter(), response) {
            (None, LLMAnswer::External) | (Some(_), LLMAnswer::External) => {}
            (None, LLMAnswer::Text(response)) => Self::print_plain(response),
            (Some(frontmatter), LLMAnswer::Text(response)) => match frontmatter {
                Typed(typed_frontmatter) => match typed_frontmatter.output() {
                    RequestedOutputFormat::JSON => {
                        Self::print_with_highlight(response, RequestedOutputFormat::JSON)
                    }
                    RequestedOutputFormat::YAML => {
                        Self::print_with_highlight(response, RequestedOutputFormat::YAML)
                    }
                    RequestedOutputFormat::Plain => Self::print_plain(response),
                    RequestedOutputFormat::Markdown => Self::print_markdown(response),
                },
                FrondmatterKind::Untyped(_) => Self::print_plain(response),
                FrondmatterKind::Raw(_) => Self::print_plain(response),
            },
        }
    }

    fn handle_ask_model(config: Option<Config>) -> anyhow::Result<()> {
        let config = Self::ensure_config(config)?;
        let provider = Self::select_provider_kind(&config)?;
        let template = Self::select_template(&config)?;
        let asker = Asker::new(template.iter());
        let answers = asker.ask()?;
        let prompt = template.compile(answers)?;

        let prompt_text = Self::prepare_prompt(&prompt);

        let response = match provider {
            LLMProviderKind::Ollama(ollama_provider) => ollama_provider.ask(prompt_text),
            LLMProviderKind::Openrouter(openrouter_provider) => {
                openrouter_provider.ask(prompt_text)
            }
            LLMProviderKind::DuckDuckGo(duckduckgo_provider) => {
                duckduckgo_provider.ask(prompt_text)
            }
        }?;

        Self::handle_prompt_response(&prompt, &response);

        Ok(())
    }
}

impl Controller for IndexController {
    fn handle(&self) -> anyhow::Result<()> {
        let config = Config::try_read();
        match config {
            Ok(config) => Self::handle_ask_model(Some(config)),
            Err(_) => {
                OnboardController::new().handle()?;
                Self::handle_ask_model(None)
            }
        }
    }
}
