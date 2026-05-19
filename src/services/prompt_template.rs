use std::{collections::HashMap, sync::Arc};

use handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderErrorReason, Template,
    template::{HelperTemplate, TemplateElement},
};
use serde_json::Value;

use crate::{entities::placeholder::Placeholder, utils::ordered_set::OrderedSet};

static UNRECOGNIZED_ERROR: &'static str = "Unrecognized";
static KEY_PROMPT: &'static str = "prompt";
static KEY_MULTILINE: &'static str = "multiline";

pub struct PromptTemplate {
    vars: OrderedSet<Placeholder>,
    template: String,
}

impl TryFrom<&str> for PromptTemplate {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let template = Template::compile(value)?;
        let mut set = OrderedSet::with_capacity(5);
        for item in template.elements.iter() {
            match item {
                TemplateElement::Expression(helper_template) => {
                    let placeholder = Self::parse_placeholder(helper_template)?;
                    set.insert(placeholder);
                }
                TemplateElement::RawString(_) => continue,
                TemplateElement::HtmlExpression(helper_template) => {
                    let name = helper_template
                        .name
                        .as_name()
                        .unwrap_or_else(|| UNRECOGNIZED_ERROR);
                    anyhow::bail!(format!(
                        "Template does not support HTML expressions: \"{name}\"",
                    ));
                }
                TemplateElement::HelperBlock(helper_template) => {
                    let name = helper_template
                        .name
                        .as_name()
                        .unwrap_or_else(|| UNRECOGNIZED_ERROR);
                    anyhow::bail!(format!(
                        "Template does not support helper blocks: \"{name}\"",
                    ))
                }
                TemplateElement::DecoratorExpression(decorator_template) => {
                    let name = decorator_template
                        .name
                        .as_name()
                        .unwrap_or_else(|| UNRECOGNIZED_ERROR);
                    anyhow::bail!(format!(
                        "Template does not support decorator expressions: \"{name}\"",
                    ))
                }
                TemplateElement::DecoratorBlock(decorator_template) => {
                    let name = decorator_template
                        .name
                        .as_name()
                        .unwrap_or_else(|| UNRECOGNIZED_ERROR);
                    anyhow::bail!(format!(
                        "Template does not support decorator blocks: \"{name}\"",
                    ))
                }
                TemplateElement::PartialExpression(decorator_template) => {
                    let name = decorator_template
                        .name
                        .as_name()
                        .unwrap_or_else(|| UNRECOGNIZED_ERROR);
                    anyhow::bail!(format!(
                        "Template does not support partial expressions: \"{name}\"",
                    ))
                }
                TemplateElement::PartialBlock(decorator_template) => {
                    let name = decorator_template
                        .name
                        .as_name()
                        .unwrap_or_else(|| UNRECOGNIZED_ERROR);
                    anyhow::bail!(format!(
                        "Template does not support partial blocks: \"{name}\""
                    ))
                }
                TemplateElement::Comment(comment) => {
                    anyhow::bail!(format!("Template does not support comments: \"{comment}\""))
                }
                _ => anyhow::bail!("Template contains unsupported syntax"),
            }
        }

        let result = Self {
            vars: set,
            template: value.to_string(),
        };

        Ok(result)
    }
}

impl PromptTemplate {
    fn validate_options(item: &HelperTemplate) -> anyhow::Result<()> {
        const ALLOWED_KEYS: [&str; 2] = [KEY_MULTILINE, KEY_PROMPT];

        for key in item.hash.keys() {
            if !ALLOWED_KEYS.contains(&key.as_str()) {
                anyhow::bail!(
                    "unknown placeholder option `{}`. Allowed options: {}",
                    key,
                    ALLOWED_KEYS.join(", ")
                );
            }
        }

        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Placeholder> {
        self.vars.iter()
    }

    fn parse_placeholder(item: &Box<HelperTemplate>) -> anyhow::Result<Placeholder> {
        Self::validate_options(item.as_ref())?;

        let name = Self::parse_name(item.as_ref())?;

        if !item.params.is_empty() {
            anyhow::bail!("placeholder `{}` must not have positional arguments", name);
        }

        let multiline = Self::parse_is_multiline(item.as_ref())?;
        let prompt = Self::parse_prompt(item.as_ref())?;

        Ok(Placeholder::new(name, multiline.unwrap_or(false), prompt))
    }

    fn parse_name(item: &HelperTemplate) -> anyhow::Result<String> {
        item.name
            .as_name()
            .map(String::from)
            .ok_or_else(|| anyhow::Error::msg("No name"))
    }

    fn parse_prompt(item: &HelperTemplate) -> anyhow::Result<Option<String>> {
        match item.hash.get(KEY_PROMPT) {
            Some(p) => match p {
                handlebars::template::Parameter::Literal(value) => match value {
                    serde_json::Value::String(prompt) => Ok(Some(prompt.to_string())),
                    _ => anyhow::bail!("\"prompt\" parameter must be of string type"),
                },
                _ => anyhow::bail!("\"prompt\" parameter must be of string type"),
            },
            _ => Ok(None),
        }
    }

    fn parse_is_multiline(item: &HelperTemplate) -> anyhow::Result<Option<bool>> {
        match item.hash.get(KEY_MULTILINE) {
            Some(p) => match p {
                handlebars::template::Parameter::Literal(value) => match value {
                    serde_json::Value::Bool(v) => Ok(Some(*v)),
                    _ => anyhow::bail!("\"multiline\" parameter must be of bool type"),
                },
                _ => anyhow::bail!("\"multiline\" parameter must be of bool type"),
            },
            None => Ok(None),
        }
    }

    pub fn compile(&self, fill_with: HashMap<Placeholder, String>) -> anyhow::Result<String> {
        let mut hbs = Handlebars::new();
        let shared_responses = Arc::new(fill_with);

        hbs.register_escape_fn(handlebars::no_escape);

        for (placeholder, _) in shared_responses.iter() {
            let placeholder_copy = placeholder.clone();
            let shared_responses = shared_responses.clone();
            let handler = Box::new(
                move |_: &Helper<'_>,
                      _: &Handlebars<'_>,
                      _: &Context,
                      _: &mut RenderContext<'_, '_>,
                      out: &mut dyn Output|
                      -> HelperResult {
                    let value = shared_responses.get(&placeholder_copy).ok_or_else(|| {
                        RenderErrorReason::Other(format!(
                            "missing value for placeholder `{}`",
                            placeholder_copy.name()
                        ))
                    })?;

                    out.write(value)?;
                    Ok(())
                },
            );
            hbs.register_helper(placeholder.name(), handler);
        }

        let data = Value::Object(
            shared_responses
                .iter()
                .map(|(key, value)| (key.name().to_owned(), Value::String(value.clone())))
                .collect(),
        );

        let rendered = hbs.render_template(self.template.as_str(), &data)?;

        Ok(rendered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::placeholder::Placeholder;

    const PROMPT: &str = "
        You are {{ role }},
        You must do {{ action prompt = \"Action:\" }}
        Context: {{ context multiline = true prompt = \"Context:\" }}
    ";

    #[test]
    fn test_parse_prompt() {
        let template = PromptTemplate::try_from(PROMPT).unwrap();
        let vars: Vec<&Placeholder> = template.iter().collect();
        assert_eq!(vars.len(), 3);

        let (first, second, third) = (
            vars.get(0).unwrap(),
            vars.get(1).unwrap(),
            vars.get(2).unwrap(),
        );

        assert_eq!(first.name(), "role");
        assert_eq!(first.multiline(), false);
        assert_eq!(first.prompt(), None);

        assert_eq!(second.name(), "action");
        assert_eq!(second.multiline(), false);
        assert_eq!(second.prompt(), Some("Action:"));

        assert_eq!(third.name(), "context");
        assert_eq!(third.multiline(), true);
        assert_eq!(third.prompt(), Some("Context:"));
    }
}
