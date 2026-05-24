use spinners::{Spinner, Spinners};

pub(crate) fn with_spinner<T>(message: impl AsRef<str>, doer: impl FnOnce() -> T) -> T {
    let mut sp = Spinner::new(Spinners::Dots9, "Thinking...".into());
    let result = doer();
    sp.stop_with_message(message.as_ref().to_string());

    result
}
