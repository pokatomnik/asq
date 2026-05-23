use std::cell::OnceCell;

use serde::Deserialize;

use crate::entities::requested_output_format::RequestedOutputFormat;

static FRONTMATTER_MARKER: &'static str = "---";

pub(crate) struct Prompt {
    raw: String,
    parsed: OnceCell<(Option<FrondmatterKind>, String)>,
}

#[derive(Clone, Deserialize)]
pub(crate) struct TypedFrontmatter {
    output: RequestedOutputFormat,
}

impl TypedFrontmatter {
    pub fn output(&self) -> &RequestedOutputFormat {
        &self.output
    }
}

pub enum FrondmatterKind {
    Typed(TypedFrontmatter),
    Untyped(yaml_serde::Value),
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

        let typed_frontmatter = yaml_serde::from_str::<TypedFrontmatter>(frontmatter_raw.as_str());

        if let Ok(typed_frontmatter) = typed_frontmatter {
            return (Some(FrondmatterKind::Typed(typed_frontmatter)), prompt);
        }

        let untyped_frontmatter =
            yaml_serde::from_str::<yaml_serde::Value>(frontmatter_raw.as_str());

        if let Ok(untyped_frontmatter) = untyped_frontmatter {
            return (Some(FrondmatterKind::Untyped(untyped_frontmatter)), prompt);
        }

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
