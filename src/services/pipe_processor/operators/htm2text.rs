use std::sync::Arc;

use dom_smoothie::{Config, Readability, TextMode};

use crate::services::pipe_processor::pipe_operator::PipeOperator;

pub(crate) struct HTM2Text {
    config: Arc<Config>,
}

impl HTM2Text {
    fn get_default_config() -> Config {
        Config {
            text_mode: TextMode::Markdown,
            ..Default::default()
        }
    }

    pub fn new(config: Option<Arc<Config>>) -> Self {
        Self {
            config: config.unwrap_or_else(|| Arc::new(Self::get_default_config())),
        }
    }
}

impl PipeOperator for HTM2Text {
    fn handle(&self, input: &str) -> anyhow::Result<String> {
        let config_clone = (*self.config).clone();
        let result: String = Readability::new(input, None, Some(config_clone))?
            .parse()?
            .text_content
            .try_into()?;

        Ok(result)
    }
}
