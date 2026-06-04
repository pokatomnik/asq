use std::fmt::Display;
use std::path::Path;
use std::{path::PathBuf, sync::Arc};

use ignore::WalkBuilder;

use crate::services::pipe_processor::pipe_operator::PipeOperator;
use crate::services::template_env::TemplateEnv;
use crate::utils::file_picker::FilePicker as FilePickerUtil;

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
            buf.push_str(format!("### {}", path.to_string_lossy().to_string()).as_str());
            buf.push_str(format!("```\n{contents}\n```").as_str());
        }

        return buf;
    }

    fn format_file_as_markdown(
        file_path: impl AsRef<Path>,
        source_code: impl AsRef<str>,
    ) -> String {
        let file_path = file_path.as_ref().to_string_lossy().to_string();
        let source_code = source_code.as_ref();
        format!("### {file_path}\n```\n{source_code}\n```\n")
    }

    fn pick_files_with_picker(&self, prompt: &str) -> anyhow::Result<String> {
        let mut files = Vec::<(PathBuf, String)>::new();
        let mut paths = std::collections::HashSet::new();

        let mut proceed = true;
        let root = self
            .template_env
            .cwd()
            .ok_or_else(|| anyhow::Error::msg("Failed to get current directory"))?;

        while proceed {
            let files_str = files
                .iter()
                .filter_map(|(p, _)| p.file_name())
                .map(|fname| fname.to_string_lossy().to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let prompt = format!("{prompt}. Files included: [{files_str}]");

            let file = dialoguer::FuzzySelect::new()
                .highlight_matches(true)
                .clear(true)
                .report(false)
                .pick_file(prompt.as_str(), root.as_path(), |_| true)?;
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

    fn pick_all(&self, _: &str) -> anyhow::Result<String> {
        let file_paths = self.get_file_paths()?;
        if file_paths.is_empty() {
            anyhow::bail!("No files there");
        }
        let mut buf = "## Project files\n".to_string();
        for selected_file in file_paths.iter() {
            let contents = std::fs::read_to_string(selected_file)?;
            let markdown = Self::format_file_as_markdown(selected_file, contents.as_str());
            buf.push_str(markdown.as_str());
        }

        Ok(buf)
    }

    fn pick_files_by_list(&self, prompt: &str) -> anyhow::Result<String> {
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

    fn select_file_picker_mode() -> anyhow::Result<FilePickerMode> {
        let choices = vec![
            FilePickerMode::ByPicker,
            FilePickerMode::ByList,
            FilePickerMode::Yolo,
        ];

        let choice_idx = dialoguer::Select::new()
            .with_prompt("Select files picker mode")
            .clear(true)
            .report(false)
            .items(choices.as_slice())
            .default(0)
            .interact()?;

        let choice = choices
            .get(choice_idx)
            .ok_or_else(|| anyhow::Error::msg("Must choose file picker mode"))?;

        Ok(*choice)
    }
}

impl PipeOperator for FilePicker {
    fn handle(&self, source: &str) -> anyhow::Result<String> {
        let choice = Self::select_file_picker_mode()?;
        match choice {
            FilePickerMode::ByPicker => Ok(self.pick_files_with_picker(source)?),
            FilePickerMode::ByList => Ok(self.pick_files_by_list(source)?),
            FilePickerMode::Yolo => Ok(self.pick_all(source)?),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum FilePickerMode {
    ByPicker,
    ByList,
    Yolo,
}

impl Display for FilePickerMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilePickerMode::ByPicker => f.write_str("Use file picker"),
            FilePickerMode::ByList => f.write_str("Show all files and pick multiple"),
            FilePickerMode::Yolo => f.write_str("Select everything except ignored"),
        }
    }
}
