use std::cell::OnceCell;

static FRONTMATTER_MARKER: &'static str = "---";

pub(crate) struct Prompt {
    raw: String,
    parsed: OnceCell<(Option<FrondmatterKind>, String)>,
}

pub enum FrondmatterKind {
    Raw(String),
}

impl Prompt {
    pub fn new(source: impl AsRef<str>) -> Self {
        Self {
            raw: source.as_ref().to_string(),
            parsed: OnceCell::new(),
        }
    }

    fn split_prompt(&self) -> (Option<String>, String) {
        let mut frontmatter_lines = Vec::new();
        let mut prompt_lines = Vec::new();
        let mut is_frontmatter = false;
        for line in self.raw.lines() {
            match (is_frontmatter, line) {
                (_, x) if x == FRONTMATTER_MARKER => {
                    is_frontmatter = !is_frontmatter;
                }
                (true, line) => {
                    frontmatter_lines.push(line);
                }
                (false, line) => prompt_lines.push(line),
            }
        }

        match frontmatter_lines.is_empty() {
            true => (None, prompt_lines.join("\n")),
            false => (Some(frontmatter_lines.join("\n")), prompt_lines.join("\n")),
        }
    }

    fn parse(&self) -> (Option<FrondmatterKind>, String) {
        let (maybe_frontmatter, prompt) = self.split_prompt();
        let Some(frontmatter_raw) = maybe_frontmatter else {
            return (None, prompt);
        };

        if frontmatter_raw.trim().len() > 0 {
            return (
                Some(FrondmatterKind::Raw(frontmatter_raw.to_string())),
                prompt,
            );
        }

        (None, prompt)
    }

    fn parse_memoized(&self) -> &(Option<FrondmatterKind>, String) {
        self.parsed.get_or_init(|| self.parse())
    }

    pub fn prompt(&self) -> &str {
        self.parse_memoized().1.as_str()
    }

    pub fn frontmatter(&self) -> Option<&FrondmatterKind> {
        self.parse_memoized().0.as_ref()
    }
}
