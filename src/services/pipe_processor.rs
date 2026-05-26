use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use crate::services::template_env::TemplateEnv;

pub(crate) type PipeOperator = Arc<dyn Fn(&str, Arc<TemplateEnv>) -> anyhow::Result<String>>;

#[derive(Default)]
pub(crate) struct PipeProcessor {
    operators: Arc<RwLock<HashMap<String, PipeOperator>>>,
}

impl PipeProcessor {
    pub fn register_operator(
        &self,
        name: impl Into<String>,
        operator: impl Fn(&str, Arc<TemplateEnv>) -> anyhow::Result<String> + 'static,
    ) -> anyhow::Result<()> {
        let Ok(mut operators) = self.operators.write() else {
            anyhow::bail!("Cannot lock");
        };

        operators.insert(name.into(), Arc::new(operator));

        Ok(())
    }

    pub fn get_by_name(
        &self,
        name: &str,
    ) -> anyhow::Result<Option<Arc<dyn Fn(&str, Arc<TemplateEnv>) -> anyhow::Result<String>>>> {
        let Ok(operators) = self.operators.read() else {
            anyhow::bail!("Cannot lock");
        };

        Ok(operators.get(name).map(|v| v.clone()))
    }
}
