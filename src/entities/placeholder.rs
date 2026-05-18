#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) struct Placeholder {
    name: String,
    multiline: bool,
    prompt: Option<String>,
}

impl AsRef<Placeholder> for Placeholder {
    fn as_ref(&self) -> &Placeholder {
        self
    }
}

impl Placeholder {
    pub fn new(name: impl AsRef<str>, multiline: bool, prompt: Option<impl AsRef<str>>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            multiline,
            prompt: prompt.map(|p| p.as_ref().to_string()),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn multiline(&self) -> bool {
        self.multiline
    }

    pub fn prompt(&self) -> Option<&str> {
        self.prompt.as_deref()
    }
}
