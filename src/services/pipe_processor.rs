use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

pub(crate) type PipeOperator = Arc<dyn Fn(&str) -> anyhow::Result<String>>;

pub(crate) struct PipeProcessor {
    operators: Arc<RwLock<HashMap<String, PipeOperator>>>,
}

impl PipeProcessor {
    pub fn register_operator(
        &self,
        name: impl Into<String>,
        operator: impl Fn(&str) -> anyhow::Result<String> + 'static,
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
    ) -> anyhow::Result<Option<Arc<dyn Fn(&str) -> anyhow::Result<String>>>> {
        let Ok(operators) = self.operators.read() else {
            anyhow::bail!("Cannot lock");
        };

        Ok(operators.get(name).map(|v| v.clone()))
    }

    fn lowercase(source: &str) -> anyhow::Result<String> {
        Ok(source.to_lowercase())
    }

    fn fetch(url: &str) -> anyhow::Result<String> {
        let client = reqwest::blocking::ClientBuilder::new().build()?;
        let request = client.get(url).build()?;
        let response = client.execute(request)?;
        let text = response.text()?;
        Ok(text)
    }

    fn file(url: &str) -> anyhow::Result<String> {
        let str = std::fs::read_to_string(url)?;
        Ok(str)
    }

    fn prompt(prompt: &str) -> anyhow::Result<String> {
        let result = dialoguer::Input::new()
            .with_prompt(prompt)
            .report(false)
            .allow_empty(false)
            .interact()?;

        Ok(result)
    }
}

impl Default for PipeProcessor {
    fn default() -> Self {
        let mut operators_map =
            HashMap::<String, Arc<dyn Fn(&str) -> anyhow::Result<String>>>::new();

        operators_map.insert("lower".to_string(), Arc::new(Self::lowercase));
        operators_map.insert("fetch".to_string(), Arc::new(Self::fetch));
        operators_map.insert("file".to_string(), Arc::new(Self::file));
        operators_map.insert("prompt".to_string(), Arc::new(Self::prompt));

        Self {
            operators: Arc::new(RwLock::new(operators_map)),
        }
    }
}
