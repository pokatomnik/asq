use std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;

use dialoguer::FuzzySelect;

pub(crate) trait FilePicker {
    fn pick_file(
        &self,
        root: impl AsRef<Path>,
        filter_file: impl Fn(&Path) -> bool,
    ) -> anyhow::Result<Option<PathBuf>>;
}

enum ListItem {
    Up(PathBuf),
    Dir(PathBuf),
    File(PathBuf),
}

impl Display for ListItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListItem::Up(_) => f.write_str("⤴️\t.."),
            ListItem::Dir(path_buf) => {
                let fname = path_buf
                    .file_name()
                    .map(|v| v.to_string_lossy().to_string())
                    .unwrap_or_else(|| "[UNKNOWN]".to_string());
                f.write_str(format!("🗂️\t{fname}").as_str())
            }
            ListItem::File(path_buf) => {
                let fname = path_buf
                    .file_name()
                    .map(|v| v.to_string_lossy().to_string())
                    .unwrap_or_else(|| "[UNKNOWN]".to_string());
                f.write_str(format!("📄\t{fname}").as_str())
            }
        }
    }
}

fn get_list_items(
    is_root: bool,
    path: impl AsRef<Path>,
    filter_file: impl Fn(&Path) -> bool,
) -> anyhow::Result<Vec<ListItem>> {
    let read_dir = std::fs::read_dir(&path)?;

    let mut folders = Vec::new();
    let mut files = Vec::new();

    for item in read_dir.into_iter() {
        let Ok(item) = item else {
            continue;
        };
        let path = item.path();
        if path.is_dir() {
            folders.push(path.clone());
        }

        let is_file = path.is_file();
        let is_md = filter_file(path.as_path());

        if is_file && is_md {
            files.push(path.clone());
        }
    }
    let size = folders.len()
        + files.len()
        + match is_root {
            true => 0,
            false => 1,
        };
    let mut result = Vec::with_capacity(size);

    files.sort_by_key(|v| {
        v.file_name()
            .map(|v| v.to_string_lossy().to_string())
            .unwrap_or_default()
    });
    folders.sort_by_key(|v| {
        v.file_name()
            .map(|v| v.to_string_lossy().to_string())
            .unwrap_or_default()
    });

    if !is_root && let Some(parent) = path.as_ref().parent() {
        result.push(ListItem::Up(parent.to_path_buf()));
    }
    result.extend(folders.into_iter().map(|v| ListItem::Dir(v)));
    result.extend(files.into_iter().map(|v| ListItem::File(v)));

    Ok(result)
}

impl<'a> FilePicker for FuzzySelect<'a> {
    fn pick_file(
        &self,
        root: impl AsRef<Path>,
        filter_file: impl Fn(&Path) -> bool,
    ) -> anyhow::Result<Option<PathBuf>> {
        let initial_root = root.as_ref().to_path_buf();
        let mut current = root.as_ref().to_path_buf();
        let mut result: Option<PathBuf> = None;
        while result.is_none() {
            let dir_contents =
                get_list_items(current == initial_root, current.as_path(), &filter_file)?;
            let idx = dialoguer::FuzzySelect::new()
                .with_prompt("Pick a file")
                .items(&dir_contents)
                .default(0)
                .highlight_matches(true)
                .clear(true)
                .report(false)
                .interact()?;
            let selected = &dir_contents
                .get(idx)
                .ok_or_else(|| anyhow::Error::msg("Cannot select file"))?;
            match selected {
                ListItem::Up(path_buf) => {
                    current = path_buf.to_owned();
                }
                ListItem::Dir(path_buf) => {
                    current = path_buf.to_owned();
                }
                ListItem::File(path_buf) => {
                    result = Some(path_buf.to_owned());
                }
            }
        }

        Ok(result)
    }
}
