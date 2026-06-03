use std::sync::Arc;

use dom_smoothie::{Config, TextMode};
use reqwest::blocking::Client;

use crate::services::code_executor::CodeExecutor;
use crate::services::pipe_processor::operators::editor::Editor;
use crate::services::pipe_processor::operators::fetch::Fetch;
use crate::services::pipe_processor::operators::file::File;
use crate::services::pipe_processor::operators::file_picker::FilePicker;
use crate::services::pipe_processor::operators::files_picker::FilesPicker;
use crate::services::pipe_processor::operators::htm2text::HTM2Text;
use crate::services::pipe_processor::operators::input::Input;
use crate::services::pipe_processor::operators::lowercase::Lowercase;
use crate::services::pipe_processor::operators::multiselect::Multiselect;
use crate::services::pipe_processor::operators::password::Password;
use crate::services::pipe_processor::operators::remember::Remember;
use crate::services::pipe_processor::operators::select::Select;
use crate::services::pipe_processor::pipe_processor::PipeProcessor;
use crate::services::template_env::TemplateEnv;
use crate::utils::describe::Describe;

static OPEN_CODE_TOKEN: char = '{';
static CLOSE_CODE_TOKEN: char = '}';

pub(crate) enum Part {
    Raw(String),
    Code(CodeExecutor),
}

impl TryFrom<Part> for String {
    type Error = anyhow::Error;

    fn try_from(value: Part) -> Result<Self, Self::Error> {
        match value {
            Part::Raw(v) => Ok(v),
            Part::Code(code_executor) => code_executor.execute(),
        }
    }
}

pub(crate) struct Parser {
    pipe_processor: Arc<PipeProcessor>,
}

impl Parser {
    fn build_pipe_processor(template_env: Arc<TemplateEnv>) -> anyhow::Result<Arc<PipeProcessor>> {
        let pipe_processor = Arc::new(PipeProcessor::default());

        let lower = Arc::new(Lowercase::new());
        let fetch = Arc::new(Fetch::new(Some(Arc::new(Client::new()))));
        let file = Arc::new(File::new(template_env.clone()));
        let input = Arc::new(Input::new());
        let editor = Arc::new(Editor::new());
        let htm2text = Arc::new(HTM2Text::new(Some(Arc::new(Config {
            text_mode: TextMode::Markdown,
            ..Default::default()
        }))));
        let select = Arc::new(Select::new());
        let multiselect = Arc::new(Multiselect::new());
        let password = Arc::new(Password::new());
        let file_picker = Arc::new(FilePicker::new(template_env.clone()));
        let files_picker = Arc::new(FilesPicker::new(template_env.clone()));
        let remember = Arc::new(Remember::new());

        pipe_processor.register_operator("lower", lower)?;
        pipe_processor.register_operator("fetch", fetch)?;
        pipe_processor.register_operator("file", file)?;
        pipe_processor.register_operator("input", input)?;
        pipe_processor.register_operator("editor", editor)?;
        pipe_processor.register_operator("htm2text", htm2text)?;
        pipe_processor.register_operator("select", select)?;
        pipe_processor.register_operator("multiselect", multiselect)?;
        pipe_processor.register_operator("password", password)?;
        pipe_processor.register_operator("file_picker", file_picker)?;
        pipe_processor.register_operator("files_picker", files_picker)?;
        pipe_processor.register_operator("remember", remember)?;

        Ok(pipe_processor)
    }

    pub fn try_create(template_env: Arc<TemplateEnv>) -> anyhow::Result<Self> {
        let pipe_processor = Self::build_pipe_processor(template_env.clone())?;
        Ok(Self { pipe_processor })
    }

    fn tokenize(&self, source: &str) -> anyhow::Result<Vec<Part>> {
        let mut bracers_guard = BracesGuard::new(2);
        let mut result = Vec::new();
        let mut buf = String::new();
        for ch in source.chars().into_iter() {
            match (ch, bracers_guard.open_state()) {
                (ch, bracers_state) if ch == OPEN_CODE_TOKEN => match (bracers_state, buf.len()) {
                    (BracersState::Open, _) => anyhow::bail!("Incorrect template"),
                    (BracersState::Intermediate, _) => {
                        bracers_guard.open()?;
                        continue;
                    }
                    (BracersState::Closed, len) if len == 0 => {
                        bracers_guard.open()?;
                        continue;
                    }
                    (BracersState::Closed, _) => {
                        result.push(Part::Raw(buf));
                        buf = String::new();
                        bracers_guard.open()?;
                        continue;
                    }
                },
                (ch, bracers_state) if ch == CLOSE_CODE_TOKEN => match (bracers_state, buf.len()) {
                    (BracersState::Closed, _) => anyhow::bail!("Incorrect template"),
                    (BracersState::Intermediate, _) => {
                        bracers_guard.close()?;
                        continue;
                    }
                    (BracersState::Open, len) if len == 0 => {
                        bracers_guard.close()?;
                        continue;
                    }
                    (BracersState::Open, _) => {
                        let pipe_processor = self.pipe_processor.clone();
                        let code_executor = CodeExecutor::new(pipe_processor, buf);
                        buf = String::new();
                        result.push(Part::Code(code_executor));
                        bracers_guard.close()?;
                        continue;
                    }
                },
                (ch, _) => {
                    buf.push(ch);
                    continue;
                }
            }
        }

        if buf.len() > 0 {
            result.push(Part::Raw(buf));
        }
        Ok(result)
    }

    pub fn compile(&self, source: impl AsRef<str>) -> anyhow::Result<String> {
        let tokens = self.tokenize(source.as_ref())?;
        let mut result = String::with_capacity(source.as_ref().len());
        for token in tokens {
            match token {
                Part::Raw(contents) => result.push_str(contents.as_str()),
                Part::Code(code_executor) => result.push_str(code_executor.execute()?.as_str()),
            }
        }

        Ok(result)
    }
}

#[derive(Clone, Copy, Debug)]
enum BracersState {
    Open,
    Closed,
    Intermediate,
}

#[derive(Debug)]
struct BracesGuard {
    open: usize,
    max: usize,
}

impl BracesGuard {
    fn new(max: usize) -> Self {
        Self { open: 0, max }
    }

    pub fn open_state(&self) -> BracersState {
        match self.open {
            open if open == 0 => BracersState::Closed,
            open if open == self.max => BracersState::Open,
            _ => BracersState::Intermediate,
        }
    }

    pub fn open(&mut self) -> anyhow::Result<()> {
        let next = self.open + 1;
        if next > self.max {
            anyhow::bail!("Incorrect open tags")
        }
        self.open = next;
        Ok(())
    }

    pub fn close(&mut self) -> anyhow::Result<()> {
        if self.open == 0 {
            anyhow::bail!("Incorrect close tags")
        }

        self.open -= 1;
        Ok(())
    }
}

impl Describe for Parser {
    fn describe(&self) -> String {
        self.pipe_processor.describe()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_compile_prefix() {
        let template_env = Arc::new(TemplateEnv::new(PathBuf::default()));
        let parser = Parser::try_create(template_env).unwrap();
        let result = parser
            .compile("{{ \"SMART ASS\" | lower }} said: fuck you")
            .unwrap();

        assert_eq!(result, "smart ass said: fuck you");
    }

    #[test]
    fn test_compile_suffix() {
        let template_env = Arc::new(TemplateEnv::new(PathBuf::default()));
        let parser = Parser::try_create(template_env).unwrap();
        let result = parser.compile("hello, {{ WORLD | lower }}").unwrap();

        assert_eq!(result, "hello, world")
    }

    #[test]
    fn test_compile_prefix_suffix() {
        let template_env = Arc::new(TemplateEnv::new(PathBuf::default()));
        let parser = Parser::try_create(template_env).unwrap();
        let result = parser
            .compile("Hi, this is {{ SHIT | lower }} around here")
            .unwrap();
        assert_eq!(result, "Hi, this is shit around here");
    }

    #[test]
    fn test_incorrect_bracers() {
        let template_env = Arc::new(TemplateEnv::new(PathBuf::default()));
        let parser = Parser::try_create(template_env).unwrap();
        let result = parser.compile("Hi, this is {{ { SHIT | lower }} around here");
        assert_eq!(result.is_err(), true)
    }

    #[test]
    fn test_nested_bracers() {
        let template_env = Arc::new(TemplateEnv::new(PathBuf::default()));
        let parser = Parser::try_create(template_env).unwrap();
        let result = parser.compile("Hi, this is {{ SHIT | lower {{ WTF }} }} around here");
        assert_eq!(result.is_err(), true)
    }

    #[test]
    fn test_no_spaces() {
        let template_env = Arc::new(TemplateEnv::new(PathBuf::default()));
        let parser = Parser::try_create(template_env).unwrap();
        let result = parser
            .compile("Hi, this is {{SHIT|lower}} around here")
            .unwrap();
        assert_eq!(result, "Hi, this is shit around here")
    }
}
