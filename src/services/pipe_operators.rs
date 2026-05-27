use std::{path::PathBuf, sync::Arc};

use dom_smoothie::{Config, Readability, TextMode};

use crate::services::{parser::Parser, template_env::TemplateEnv};

/// Returns a lowercase copy of the provided string slice.
///
/// # Arguments
///
/// * `source` - The input string to convert to lowercase.
/// * `template_env` - The template environment used to resolve relative paths.
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
/// * `template_env` - The template environment used to resolve relative paths.
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

/// Reads the contents of a file as a string.
///
/// # Arguments
///
/// * `path` - The file path to read. If relative, it is resolved against the template's prompt directory.
/// * `template_env` - The template environment used to resolve relative paths.
///
/// # Returns
///
/// A `String` containing the file contents.
///
/// # Examples
///
/// ```
/// let result = file("example.txt", template_env).unwrap();
/// assert!(!result.is_empty());
/// ```
pub(crate) fn file(path: &str, template_env: Arc<TemplateEnv>) -> anyhow::Result<String> {
    let path = PathBuf::from(path);

    if path.is_absolute() {
        let result = std::fs::read_to_string(path)?;
        return Ok(result);
    }

    let prompt_dir = template_env
        .prompt_dir()
        .ok_or_else(|| anyhow::Error::msg("No prompt dir"))?
        .to_path_buf();
    let required_path = prompt_dir.join(path);
    let template_contents = std::fs::read_to_string(required_path.as_path())?;

    let template_env = Arc::new(TemplateEnv::new(required_path));
    let parser = Parser::try_create(template_env)?;
    let prompt_str = parser.compile(template_contents)?;

    Ok(prompt_str)
}

/// Prompts the user for input with the provided message.
///
/// # Arguments
///
/// * `prompt` - The message displayed to the user.
/// * `template_env` - The template environment used to resolve relative paths.
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
/// * `template_env` - The template environment used to resolve relative paths.
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
/// * `template_env` - The template environment used to resolve relative paths.
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
