use reqwest::blocking::RequestBuilder;

pub(crate) trait RequestBuilderExt {
    fn with_optional_bearer_token<T>(self, token: Option<T>) -> Self
    where
        T: AsRef<str>;

    fn application_json(self) -> Self;
}

impl RequestBuilderExt for RequestBuilder {
    fn with_optional_bearer_token<T>(self, token: Option<T>) -> Self
    where
        T: AsRef<str>,
    {
        let Some(token) = token else {
            return self;
        };

        let token = token.as_ref();
        self.header("Authorization", format!("Bearer {token}"))
    }

    fn application_json(self) -> Self {
        self.header("Content-Type", "application/json")
    }
}
