use crate::tools::{
    memory::{Memory, MemoryExecuteArg},
    tool::Tool,
};

pub(crate) struct ToolExecutor {
    memory: Memory,
}

impl ToolExecutor {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
        }
    }

    fn save_memory(&self, memory_execute_arg: &MemoryExecuteArg) -> anyhow::Result<String> {
        self.memory.execute(memory_execute_arg)
    }

    pub fn call_by_name(&self, name: &str, args_json: &str) -> anyhow::Result<String> {
        if name == Memory::name()
            && let Ok(memory_arg) = serde_json::from_str::<MemoryExecuteArg>(args_json)
        {
            return self.save_memory(&memory_arg);
        }

        anyhow::bail!("Failed to execute tools")
    }
}
