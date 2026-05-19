pub(crate) trait Controller {
    fn handle(&self) -> anyhow::Result<()>;
}
