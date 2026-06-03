use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use ignore::WalkBuilder;

use crate::services::{pipe_processor::pipe_operator::PipeOperator, template_env::TemplateEnv};

pub(crate) struct FilesPicker {
    template_env: Arc<TemplateEnv>,
}

impl FilesPicker {
    pub fn new(template_env: Arc<TemplateEnv>) -> Self {
        Self { template_env }
    }

    fn format_file_as_markdown(
        file_path: impl AsRef<Path>,
        source_code: impl AsRef<str>,
    ) -> String {
        let file_path = file_path.as_ref().to_string_lossy().to_string();
        let source_code = source_code.as_ref();
        format!("### {file_path}\n```\n{source_code}\n```\n")
    }

    fn get_file_paths(&self) -> anyhow::Result<Vec<PathBuf>> {
        let mut result = Vec::new();
        let cwd = self
            .template_env
            .cwd()
            .ok_or_else(|| anyhow::Error::msg("Failed to get current dir"))?;
        let iterator = WalkBuilder::new(cwd)
            .hidden(true)
            .git_exclude(true)
            .git_ignore(true)
            .follow_links(false)
            .build();

        for dir_entry_result in iterator {
            let Ok(dir_entry_result) = dir_entry_result else {
                continue;
            };
            let Ok(metadata) = dir_entry_result.metadata() else {
                continue;
            };

            if !metadata.is_file() {
                continue;
            }

            result.push(dir_entry_result.into_path());
        }

        Ok(result)
    }
}

impl PipeOperator for FilesPicker {
    fn handle(&self, prompt: &str) -> anyhow::Result<String> {
        let file_paths = self.get_file_paths()?;
        if file_paths.is_empty() {
            anyhow::bail!("No files there");
        }
        let file_paths_as_strings = file_paths
            .iter()
            .map(|v| v.to_string_lossy().to_string())
            .collect::<Vec<String>>();
        let selected_indexes = dialoguer::MultiSelect::new()
            .items(file_paths_as_strings.as_slice())
            .max_length(20)
            .with_prompt(prompt)
            .clear(true)
            .report(false)
            .interact()?;

        let mut selected_file_paths = Vec::with_capacity(file_paths.len());
        for index in selected_indexes {
            if let Some(file) = file_paths.get(index) {
                selected_file_paths.push(file);
            }
        }

        if selected_file_paths.is_empty() {
            anyhow::bail!("No files selected");
        }

        let mut buf = "## Project files\n".to_string();
        for selected_file in selected_file_paths {
            let contents = std::fs::read_to_string(selected_file)?;
            let markdown = Self::format_file_as_markdown(selected_file, contents.as_str());
            buf.push_str(markdown.as_str());
        }

        Ok(buf)
    }
}
