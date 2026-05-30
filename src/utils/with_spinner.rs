use spinners::{Spinner, Spinners};

pub(crate) fn with_spinner<T>(
    start_message: impl AsRef<str>,
    stop_message: impl AsRef<str>,
    doer: impl FnOnce() -> T,
) -> T {
    let mut sp = Spinner::new(Spinners::Dots9, start_message.as_ref().into());
    let result = doer();
    sp.stop_with_message(stop_message.as_ref().to_string());

    result
}
