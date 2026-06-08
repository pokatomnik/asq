use std::fmt::Display;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::utils::fileman::FileMan;

pub struct Memory;

static MEMORY_FNAME: &'static str = "memory.json";
static MEMORY_HEADER: &'static str = "User memories";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Memo {
    date_added: String,
    text: String,
}

impl Memory {
    pub fn new() -> Self {
        Self {}
    }

    fn now() -> String {
        Utc::now().to_rfc3339()
    }

    fn memory_header() -> String {
        format!("## {MEMORY_HEADER}:")
    }

    fn get_memos() -> Vec<Memo> {
        let Ok(contents) = String::from_utf8(
            FileMan::read_data(MEMORY_FNAME)
                .map(|v| v.as_ref().to_vec())
                .unwrap_or_default(),
        ) else {
            return vec![];
        };
        match serde_json::from_str::<Vec<Memo>>(contents.as_str()) {
            Ok(v) => v,
            Err(_) => vec![],
        }
    }

    pub fn get_memories() -> Vec<String> {
        let memories = Self::get_memos();
        memories.into_iter().map(|v| v.text).collect()
    }

    pub fn replace_memories(memories: Vec<&str>) -> anyhow::Result<()> {
        FileMan::save_data(MEMORY_FNAME, memories.join("\n"))
    }

    pub fn add_memory(memory: impl AsRef<str>) -> anyhow::Result<()> {
        let text = memory.as_ref().to_string();
        let mut memories = Self::get_memos();

        // Prevent duplicates
        for curr in memories.iter() {
            if curr.text.as_str() == text.as_str() {
                return Ok(());
            }
        }

        memories.push(Memo {
            text,
            date_added: Self::now(),
        });
        let memos_json = serde_json::to_string_pretty(&memories)?;
        FileMan::save_data(MEMORY_FNAME, memos_json)
    }
}

impl Display for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let memories = Self::get_memories();
        if memories.is_empty() {
            return Ok(());
        }

        let mut buf = String::with_capacity(2048);
        buf.push_str(&Self::memory_header());
        buf.push('\n');

        for memory in memories.iter() {
            buf.push_str(format!("- {memory}\n").as_str());
        }

        f.write_str(buf.as_str())
    }
}
