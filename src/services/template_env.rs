use std::path::{Path, PathBuf};

pub struct TemplateEnv {
    prompt_path: PathBuf,
}

impl TemplateEnv {
    pub fn new(prompt_path: impl Into<PathBuf>) -> Self {
        let prompt_path = prompt_path.into();
        Self { prompt_path }
    }

    pub fn prompt_path(&self) -> &Path {
        self.prompt_path.as_path()
    }

    pub fn prompt_dir(&self) -> Option<&Path> {
        self.prompt_path.parent()
    }
}
