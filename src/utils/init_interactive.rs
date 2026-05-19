pub(crate) trait InitInteractive<T> {
    fn init_interactive() -> anyhow::Result<T>;
}
