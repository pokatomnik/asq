use std::sync::Arc;

use reqwest::blocking::Client;

use crate::services::pipe_processor::pipe_operator::PipeOperator;

pub(crate) struct Fetch {
    client: anyhow::Result<Arc<Client>>,
}

impl Fetch {
    fn build_client() -> anyhow::Result<Arc<Client>> {
        let client = reqwest::blocking::ClientBuilder::new().build()?;
        Ok(Arc::new(client))
    }

    pub fn new(client: Option<Arc<Client>>) -> Self {
        if let Some(client) = client {
            return Self { client: Ok(client) };
        }
        let client_result = Self::build_client();
        Self {
            client: client_result,
        }
    }
}

impl PipeOperator for Fetch {
    fn handle(&self, url: &str) -> anyhow::Result<String> {
        let Ok(ref client) = self.client else {
            anyhow::bail!("Failed to initialize HTTP client")
        };

        let request = client.get(url).build()?;
        let response = client.execute(request)?;
        let text = response.text()?;
        Ok(text)
    }
}
