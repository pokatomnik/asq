use std::sync::OnceLock;

use crate::{
    entities::{message::Message, role::Role, system_prompt::SYSTEM_PROMPT},
    utils::fileman::FileMan,
};

pub struct History {
    use_history: bool,
    history_messages: OnceLock<Vec<Message>>,
}

static HISTORY_FILE_NAME: &'static str = "chat.json";

impl History {
    pub fn new(use_history: bool) -> Self {
        Self {
            use_history,
            history_messages: Default::default(),
        }
    }

    fn get_history_messages(&self) -> Vec<Message> {
        let Ok(json) = FileMan::read_data(HISTORY_FILE_NAME) else {
            return Vec::default();
        };
        serde_json::from_slice::<Vec<Message>>(json.as_ref()).unwrap_or_default()
    }

    fn get_history_messages_once(&self) -> &Vec<Message> {
        self.history_messages
            .get_or_init(|| self.get_history_messages())
    }

    pub fn is_empty(&self) -> bool {
        self.get_history_messages().len() == 0
    }

    fn save_messages(&self, messages: Vec<Message>) -> anyhow::Result<()> {
        let messages_str = serde_json::to_string_pretty(&messages)?;

        FileMan::save_data(HISTORY_FILE_NAME, messages_str)?;

        Ok(())
    }

    pub fn with_history(
        &self,
        doer: impl FnOnce(Vec<Message>) -> anyhow::Result<Vec<Message>>,
    ) -> anyhow::Result<()> {
        let mut messages = match self.use_history {
            true => self.get_history_messages_once().clone(),
            false => Vec::with_capacity(1),
        };
        if messages.len() == 0 {
            messages = vec![Message::new(Role::System, SYSTEM_PROMPT)]
        }
        let updated_messages = doer(messages)?;

        self.save_messages(updated_messages)
    }
}
