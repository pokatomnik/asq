use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use crate::services::pipe_processor::pipe_operator::PipeOperator;
use crate::utils::describe::Describe;

#[derive(Default)]
pub(crate) struct PipeProcessor {
    operators: Arc<RwLock<HashMap<String, Arc<dyn PipeOperator>>>>,
}

impl PipeProcessor {
    pub fn register_operator(
        &self,
        name: impl Into<String>,
        operator: Arc<dyn PipeOperator>,
    ) -> anyhow::Result<()> {
        let Ok(mut operators) = self.operators.write() else {
            anyhow::bail!("Cannot lock");
        };

        operators.insert(name.into(), operator);

        Ok(())
    }

    pub fn get_by_name(&self, name: &str) -> anyhow::Result<Option<Arc<dyn PipeOperator>>> {
        let Ok(operators) = self.operators.read() else {
            anyhow::bail!("Cannot lock");
        };

        Ok(operators.get(name).map(|v| v.clone()))
    }
}

impl Describe for PipeProcessor {
    fn describe(&self) -> String {
        let Ok(operators) = self.operators.read() else {
            return String::default();
        };

        let mut buf = Vec::with_capacity(operators.len());
        for (name, _) in operators.iter() {
            buf.push(name.to_string());
        }
        buf.sort();

        format!("Supported operators: {}", buf.join(", "))
    }
}
