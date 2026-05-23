use std::collections::HashMap;

use crate::entities::placeholder::Placeholder;

pub struct Asker {
    vars: Vec<Placeholder>,
}

impl Asker {
    pub fn new(vars: impl Iterator<Item = impl AsRef<Placeholder>>) -> Self {
        let vars = vars.map(|v| v.as_ref().to_owned()).collect();
        Self { vars }
    }

    fn ask_single_line(placeholder: &Placeholder) -> anyhow::Result<String> {
        let prompt = placeholder.prompt().unwrap_or(placeholder.name());
        let input = dialoguer::Input::<String>::new()
            .with_prompt(prompt)
            .report(false)
            .allow_empty(false)
            .show_default(false)
            .default(String::default());

        let result = input.interact()?;
        Ok(result)
    }

    fn ask_multi_line(placeholder: &Placeholder) -> anyhow::Result<String> {
        let prompt = placeholder.prompt().unwrap_or(placeholder.name());
        let input = dialoguer::Editor::new().edit(prompt);
        match input {
            Ok(None) => anyhow::bail!("empty input"),
            Ok(Some(v)) => Ok(v),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }

    pub fn ask(&self) -> anyhow::Result<HashMap<Placeholder, String>> {
        let mut result = HashMap::new();
        for item in self.vars.iter() {
            let response = match item.multiline() {
                true => Self::ask_multi_line(&item),
                false => Self::ask_single_line(&item),
            }?;

            result.insert(item.clone(), response);
        }

        Ok(result)
    }
}
