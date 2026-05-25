pub(crate) fn lowercase(source: &str) -> anyhow::Result<String> {
    Ok(source.to_lowercase())
}

pub(crate) fn fetch(url: &str) -> anyhow::Result<String> {
    let client = reqwest::blocking::ClientBuilder::new().build()?;
    let request = client.get(url).build()?;
    let response = client.execute(request)?;
    let text = response.text()?;
    Ok(text)
}

pub(crate) fn file(path: &str) -> anyhow::Result<String> {
    let str = std::fs::read_to_string(path)?;
    Ok(str)
}

pub(crate) fn input(prompt: &str) -> anyhow::Result<String> {
    let result = dialoguer::Input::new()
        .with_prompt(prompt)
        .report(false)
        .allow_empty(false)
        .interact()?;

    Ok(result)
}

pub(crate) fn editor(prompt: &str) -> anyhow::Result<String> {
    let result = dialoguer::Editor::new().edit(prompt)?;
    Ok(result.unwrap_or_default())
}
