use std::sync::Arc;

use dom_smoothie::{Config, TextMode};
use reqwest::blocking::Client;

use crate::services::pipe_processor::operators::editor::Editor;
use crate::services::pipe_processor::operators::fetch::Fetch;
use crate::services::pipe_processor::operators::file::File;
use crate::services::pipe_processor::operators::file_picker::FilePicker;
use crate::services::pipe_processor::operators::htm2text::HTM2Text;
use crate::services::pipe_processor::operators::input::Input;
use crate::services::pipe_processor::operators::lowercase::Lowercase;
use crate::services::pipe_processor::operators::multiselect::Multiselect;
use crate::services::pipe_processor::operators::now::Now;
use crate::services::pipe_processor::operators::password::Password;
use crate::services::pipe_processor::operators::select::Select;
use crate::services::pipe_processor::pipe_processor::PipeProcessor;
use crate::services::template_env::TemplateEnv;

pub(crate) fn user_pipe_processor(template_env: Arc<TemplateEnv>) -> anyhow::Result<PipeProcessor> {
    let pipe_processor = PipeProcessor::default();

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
    let now = Arc::new(Now::new());

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
    pipe_processor.register_operator("now", now)?;

    Ok(pipe_processor)
}
