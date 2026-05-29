use std::{path::PathBuf, sync::Arc};

use crate::{
    services::{pipe_processor::pipe_operator::PipeOperator, template_env::TemplateEnv},
    utils::file_picker::FilePicker as FilePickerUtil,
};

pub(crate) struct FilePicker {
    template_env: Arc<TemplateEnv>,
}

impl FilePicker {
    pub fn new(template_env: Arc<TemplateEnv>) -> Self {
        Self { template_env }
    }

    fn confirm_continue() -> bool {
        dialoguer::Confirm::new()
            .default(false)
            .report(false)
            .with_prompt("Add more files?")
            .show_default(true)
            .interact()
            .unwrap_or_default()
    }

    fn build_prompt(files: impl IntoIterator<Item = (PathBuf, String)>) -> String {
        let mut buf = String::with_capacity(2048);

        for (path, contents) in files {
            buf.push_str(format!("## {}", path.to_string_lossy().to_string()).as_str());
            buf.push_str(format!("```\n{contents}\n```").as_str());
        }

        return buf;
    }
}

impl PipeOperator for FilePicker {
    fn handle(&self, prompt_base: &str) -> anyhow::Result<String> {
        let mut files = Vec::<(PathBuf, String)>::new();
        let mut paths = std::collections::HashSet::new();

        let mut proceed = true;
        let root = self
            .template_env
            .cwd()
            .ok_or_else(|| anyhow::Error::msg("Failed to get current directory"))?;

        while proceed {
            let mut prompt = prompt_base.to_string();
            if files.len() > 0 {
                prompt.push_str(format!(" ({} file(s) included)", files.len()).as_str());
            }

            let file = dialoguer::FuzzySelect::new()
                .with_prompt(prompt)
                .highlight_matches(true)
                .clear(true)
                .report(false)
                .pick_file(root.as_path(), |_| true)?;
            let Some(file_path) = file else {
                proceed = Self::confirm_continue();
                continue;
            };

            let contents = std::fs::read_to_string(&file_path)?;
            if !paths.contains(&file_path) {
                paths.insert(file_path.clone());
                files.push((file_path, contents));
            }

            proceed = Self::confirm_continue();
        }

        if files.len() == 0 {
            anyhow::bail!("No files selected");
        }

        let prompt = Self::build_prompt(files);

        Ok(prompt)
    }
}
