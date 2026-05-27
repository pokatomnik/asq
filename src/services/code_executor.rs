use std::sync::Arc;

use crate::services::pipe_processor::pipe_processor::PipeProcessor;

static SPLIT_TOKEN: char = '|';

pub(crate) struct CodeExecutor {
    pipe_processor: Arc<PipeProcessor>,
    source_raw: String,
}

impl CodeExecutor {
    pub fn new(pipe_processor: Arc<PipeProcessor>, source: impl AsRef<str>) -> Self {
        let source_raw = source.as_ref().to_string();
        Self {
            pipe_processor,
            source_raw,
        }
    }

    fn get_code_raw(code: impl AsRef<str>) -> String {
        code.as_ref().trim().to_string()
    }

    fn tokenize(&self, source: &str) -> anyhow::Result<(String, Vec<String>)> {
        let tokens = source
            .split(SPLIT_TOKEN)
            .map(|v| v.trim())
            .collect::<Vec<&str>>();
        let Some(first) = tokens.first() else {
            anyhow::bail!("First token sould be a static string")
        };

        let incorrect_token = tokens.iter().skip(1).find(|operator| {
            self.pipe_processor
                .get_by_name(operator)
                .is_ok_and(|v| v.is_none())
        });

        if incorrect_token.is_some() {
            let token = incorrect_token.map(|v| v.to_string()).unwrap_or_default();
            anyhow::bail!("Code has incorret token(s): {token}");
        }

        let start = first.to_string().trim_matches('"').to_string();
        let operators = tokens
            .into_iter()
            .skip(1)
            .map(ToString::to_string)
            .collect();

        Ok((start, operators))
    }

    /// Runs a code for template pipeline, for example:
    /// "Your name:" | prompt | lowercase
    pub fn execute(&self) -> anyhow::Result<String> {
        let code_raw = Self::get_code_raw(self.source_raw.as_str());

        let (mut current, operators) = self.tokenize(code_raw.as_str())?;
        for operator in operators {
            match self.pipe_processor.get_by_name(operator.as_str()) {
                Ok(maybe_operator) => match maybe_operator {
                    Some(operator) => {
                        current = operator.handle(current.as_str())?;
                    }
                    None => anyhow::bail!("Wrong operator: {operator}"),
                },
                Err(e) => return Err(anyhow::Error::from(e)),
            }
        }

        Ok(current)
    }
}
