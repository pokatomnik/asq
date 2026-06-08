use std::sync::OnceLock;

use crate::entities::message::Message;
use crate::entities::role::Role;
use crate::entities::system_prompt::system_prompt;
use crate::services::memory::Memory;
use crate::tools::tool::Tool;
use crate::utils::fileman::FileMan;

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

    fn get_system_prompt(&self, tools: &[Box<dyn Tool>]) -> anyhow::Result<String> {
        let memory = Memory::new();
        let mut result = system_prompt(tools);
        result.push('\n');
        let memories = memory.to_string();
        result.push_str(memories.as_str());

        Ok(result)
    }

    pub fn with_history(
        &self,
        tools: &[Box<dyn Tool>],
        doer: impl FnOnce(Vec<Message>) -> anyhow::Result<Vec<Message>>,
    ) -> anyhow::Result<()> {
        let mut messages = match self.use_history {
            true => self.get_history_messages_once().clone(),
            false => Vec::with_capacity(1),
        };
        if messages.len() == 0 {
            messages = vec![Message::new(
                Role::System,
                self.get_system_prompt(tools)?.as_str(),
            )]
        }
        let updated_messages = doer(messages)?;

        self.save_messages(updated_messages)
    }
}
