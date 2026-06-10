use anyhow::Context;

const AGENTS_DIR: &str = ".agents";
const SKILLS_SUBDIR: &str = "skills";
const SKILL_MD: &str = "SKILL.md";
use std::path::{Path, PathBuf};

pub(crate) struct Skills {
    cwd: PathBuf,
}

impl Skills {
    pub fn new(cwd: impl AsRef<Path>) -> Self {
        Self {
            cwd: cwd.as_ref().to_path_buf(),
        }
    }

    /// Prompt the user to select a skill directory name via fuzzy select.
    fn select_skill_name(&self) -> Option<String> {
        // Directory where skills are stored: <cwd>/.agents/skills
        let skills_dir = self.cwd.join(AGENTS_DIR).join(SKILLS_SUBDIR);
        // Collect the names of sub‑directories (each representing a skill)
        let skill_names: Vec<String> = std::fs::read_dir(skills_dir)
            .ok()? // read_dir may fail, treat as no skills
            .filter_map(|e| e.ok()) // unwrap DirEntry
            .filter(|e| e.path().is_dir()) // keep only directories
            .filter_map(|e| e.file_name().into_string().ok()) // convert OsString to String
            .collect();

        if skill_names.is_empty() {
            return None;
        }

        // Show fuzzy selector and return the chosen skill name
        dialoguer::FuzzySelect::new()
            .report(false)
            .clear(true)
            .with_prompt("Select skill")
            .default(0)
            .highlight_matches(true)
            .items(&skill_names)
            .interact()
            .ok()
            .and_then(|idx| skill_names.get(idx).cloned())
    }

    pub fn get_contents_interactive(&self) -> Option<String> {
        let selected = self.select_skill_name()?;
        let skill_path = self
            .cwd
            .join(AGENTS_DIR)
            .join(SKILLS_SUBDIR)
            .join(&selected)
            .join(SKILL_MD);
        let contents = std::fs::read_to_string(&skill_path).ok()?;
        Some(contents)
    }

    pub fn delete_interactive(&self) -> anyhow::Result<()> {
        let selected = self
            .select_skill_name()
            .with_context(|| "No skills selected")?;
        let dir_path = self
            .cwd
            .join(AGENTS_DIR)
            .join(SKILLS_SUBDIR)
            .join(&selected);
        std::fs::remove_dir_all(&dir_path)
            .with_context(|| format!("Failed to delete skill directory: {}", dir_path.display()))?;
        Ok(())
    }

    pub fn install(&self, name: &str, url: &str) -> anyhow::Result<()> {
        // Download skill markdown content from the provided URL
        let response =
            reqwest::blocking::get(url).with_context(|| format!("Failed to GET URL: {}", url))?;
        let content = response
            .text()
            .with_context(|| format!("Failed to read response body from URL: {}", url))?;

        // Ensure target directory exists: <cwd>/.agents/skills/{name}
        let target_dir = self.cwd.join(AGENTS_DIR).join(SKILLS_SUBDIR).join(name);
        std::fs::create_dir_all(&target_dir).with_context(|| {
            format!("Failed to create skill directory: {}", target_dir.display())
        })?;

        // Write the markdown content to SKILL.md inside the directory
        let skill_md_path = target_dir.join(SKILL_MD);
        std::fs::write(&skill_md_path, content)
            .with_context(|| format!("Failed to write skill file: {}", skill_md_path.display()))?;
        Ok(())
    }
}
