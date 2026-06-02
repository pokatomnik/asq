use std::fmt::Display;

use crate::utils::fileman::FileMan;

pub struct Memory;

static MEMORY_FNAME: &'static str = "memory.md";
static MEMORY_HEADER: &'static str = "User memories";

impl Memory {
    pub fn new() -> Self {
        Self {}
    }

    fn memory_header() -> String {
        format!("## {MEMORY_HEADER}:")
    }

    pub fn get_memories() -> anyhow::Result<Vec<String>> {
        let contents = String::from_utf8(
            FileMan::read_data(MEMORY_FNAME)
                .map(|v| v.as_ref().to_vec())
                .unwrap_or_default(),
        )?;

        let memories = contents
            .lines()
            .map(|l| l.trim())
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        Ok(memories)
    }

    pub fn replace_memories(memories: Vec<&str>) -> anyhow::Result<()> {
        FileMan::save_data(MEMORY_FNAME, memories.join("\n"))
    }

    pub fn add_memory(memory: String) -> anyhow::Result<()> {
        let mut memories = Self::get_memories()?;
        memories.push(memory);
        FileMan::save_data(MEMORY_FNAME, memories.join("\n"))
    }
}

impl Display for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let memories = Self::get_memories().map_err(|_| std::fmt::Error)?;
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
