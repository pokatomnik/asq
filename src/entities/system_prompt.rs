use crate::tools::tool::Tool;

static SYSTEM_PROMPT: &'static str = include_str!("./system_prompt.md");

pub(crate) fn system_prompt(tools: &[Box<dyn Tool>]) -> String {
    let mut buf = String::from(SYSTEM_PROMPT);

    for tool in tools {
        buf.push_str(tool.skill());
    }

    buf
}
