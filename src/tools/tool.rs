use serde_json::Value;

pub(crate) trait Tool {
    const NAME: &'static str;

    fn definition() -> anyhow::Result<Value>;

    fn exec(&self, params: &str) -> anyhow::Result<String>;
}
