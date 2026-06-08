use serde_json::Value;

pub(crate) trait Tool {
    fn name(&self) -> &'static str;

    fn skill(&self) -> &'static str;

    fn definition(&self) -> anyhow::Result<Value>;

    fn exec(&self, params: &str) -> anyhow::Result<String>;
}
