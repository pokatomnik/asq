use crate::services::{asker::Asker, prompt_template::PromptTemplate};

mod entities;
mod services;
mod utils;

const FOO: &'static str = "
    You are {{ role }},
    You must do {{ action prompt = \"Action:\" }}
    Context: {{ context multiline = true prompt = \"Context:\" }}
";

fn main() -> anyhow::Result<()> {
    let template = PromptTemplate::try_from(FOO)?;

    let asker = Asker::new(template.iter());
    let result = asker.ask();

    println!("{:#?}", result);

    Ok(())
}
