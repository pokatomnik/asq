use std::sync::Arc;

use dom_smoothie::{Config, Readability, TextMode};

use crate::services::template_env::TemplateEnv;

/// Returns a lowercase copy of the provided string slice.
///
/// # Arguments
///
/// * `source` - The input string to convert to lowercase.
///
/// # Returns
///
/// A `String` containing the lowercase version of `source`.
///
/// # Examples
///
/// ```
/// let result = lowercase("HeLLo").unwrap();
/// assert_eq!(result, "hello");
/// ```
pub(crate) fn lowercase(source: &str, _: Arc<TemplateEnv>) -> anyhow::Result<String> {
    Ok(source.to_lowercase())
}

/// Fetches the contents of the provided URL as a string.
///
/// # Arguments
///
/// * `url` - The URL to request.
///
/// # Returns
///
/// A `String` containing the response body.
///
/// # Examples
///
/// ```
/// let result = fetch("https://example.com").unwrap();
/// assert!(!result.is_empty());
/// ```
pub(crate) fn fetch(url: &str, _: Arc<TemplateEnv>) -> anyhow::Result<String> {
    let client = reqwest::blocking::ClientBuilder::new().build()?;
    let request = client.get(url).build()?;
    let response = client.execute(request)?;
    let text = response.text()?;
    Ok(text)
}

/// Returns the contents of the file at the provided path as a string.
///
/// # Arguments
///
/// * `path` - The path to the file to read.
///
/// # Returns
///
/// A `String` containing the file contents.
///
/// # Examples
///
/// ```
/// let result = file("./Cargo.toml").unwrap();
/// assert!(!result.is_empty());
/// ```
pub(crate) fn file(path: &str, _: Arc<TemplateEnv>) -> anyhow::Result<String> {
    let str = std::fs::read_to_string(path)?;
    Ok(str)
}

/// Prompts the user for input with the provided message.
///
/// # Arguments
///
/// * `prompt` - The message displayed to the user.
///
/// # Returns
///
/// A `String` containing the user's input.
///
/// # Examples
///
/// ```
/// let result = input("Enter your name:").unwrap();
/// assert!(!result.is_empty());
/// ```
pub(crate) fn input(prompt: &str, _: Arc<TemplateEnv>) -> anyhow::Result<String> {
    let result = dialoguer::Input::new()
        .with_prompt(prompt)
        .report(false)
        .allow_empty(false)
        .interact()?;

    Ok(result)
}

/// Prompts the user to edit the provided text in their configured editor.
///
/// # Arguments
///
/// * `prompt` - The initial text to display in the editor.
///
/// # Returns
///
/// A `String` containing the edited text, or an empty string if the editor was closed
/// without producing output.
///
/// # Examples
///
/// ```
/// let result = editor("Hello, world!").unwrap();
/// assert!(result.contains("Hello"));
/// ```
pub(crate) fn editor(prompt: &str, _: Arc<TemplateEnv>) -> anyhow::Result<String> {
    let result = dialoguer::Editor::new().edit(prompt)?;
    Ok(result.unwrap_or_default())
}

/// Converts HTML to text using readability parsing.
///
/// # Arguments
///
/// * `input` - The HTML content to convert.
///
/// # Returns
///
/// A `String` containing the converted text content.
///
/// # Examples
///
/// ```
/// let html = "<h1>Hello</h1><p>world</p>";
/// let result = htm2text(html).unwrap();
/// assert!(result.contains("Hello"));
/// ```
pub(crate) fn htm2text(input: &str, _: Arc<TemplateEnv>) -> anyhow::Result<String> {
    let config = Config {
        text_mode: TextMode::Markdown,
        ..Default::default()
    };
    let result: String = Readability::new(input, None, Some(config))?
        .parse()?
        .text_content
        .try_into()?;

    Ok(result)
}
