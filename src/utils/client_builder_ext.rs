use reqwest::{Proxy, blocking::ClientBuilder};

use crate::entities::proxy::LLMProxy;

pub(crate) trait ClientBuilderExt {
    fn with_optional_proxy<P>(self, proxy: Option<P>) -> Self
    where
        P: AsRef<LLMProxy>;
}

impl ClientBuilderExt for ClientBuilder {
    fn with_optional_proxy<P>(self, proxy: Option<P>) -> Self
    where
        P: AsRef<LLMProxy>,
    {
        let Some(proxy) = proxy else {
            return self;
        };

        let Ok(proxy) = Proxy::all(proxy.as_ref().proxy_scheme()) else {
            return self;
        };

        self.proxy(proxy)
    }
}
