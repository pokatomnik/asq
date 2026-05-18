use std::path::PathBuf;

pub(crate) struct FileMan;

static USER_CONFIG_DIR_NAME: &'static str = ".config";
static ASQ_DIR_NAME: &'static str = "asq";

impl FileMan {
    fn new() -> Self {
        Self
    }

    pub fn home_dir() -> Option<PathBuf> {
        std::env::home_dir()
    }

    fn resolve_asq_dir_path() -> Option<PathBuf> {
        let homedir = Self::home_dir()?;
        let dir = homedir.join(USER_CONFIG_DIR_NAME).join(ASQ_DIR_NAME);
        Some(dir)
    }

    fn ensure_asq_dir_exists() -> anyhow::Result<()> {
        let asq_dir_path = Self::resolve_asq_dir_path()
            .ok_or_else(|| anyhow::anyhow!("Cannot resolve user home directory"))?;
        std::fs::create_dir_all(asq_dir_path)?;
        Ok(())
    }

    pub fn save_data(fname: impl AsRef<str>, data: impl AsRef<[u8]>) -> anyhow::Result<()> {
        Self::ensure_asq_dir_exists()?;
        let asq_dir_path = Self::resolve_asq_dir_path()
            .ok_or_else(|| anyhow::anyhow!("Cannot resolve user home directory"))?;
        let full_file_path = asq_dir_path.join(fname.as_ref());
        std::fs::write(full_file_path, data)?;
        Ok(())
    }

    pub fn read_data(fname: impl AsRef<str>) -> anyhow::Result<impl AsRef<[u8]>> {
        let asq_dir_path = Self::resolve_asq_dir_path()
            .ok_or_else(|| anyhow::anyhow!("Cannot resolve user home directory"))?;
        let full_file_path = asq_dir_path.join(fname.as_ref());
        let data = std::fs::read(full_file_path)?;
        Ok(data)
    }
}
