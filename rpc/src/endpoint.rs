use crate::channel::SgxTrustedTlsChannel;
use crate::config::SgxTrustedTlsClientConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::prelude::v1::*;

pub struct Endpoint {
    url: String,
    config: SgxTrustedTlsClientConfig,
}

impl Endpoint {
    pub fn new(url: &str) -> Self {
        let config = SgxTrustedTlsClientConfig::new();
        Self {
            url: url.to_string(),
            config,
        }
    }

    pub fn connect<U, V>(&self) -> Result<SgxTrustedTlsChannel<U, V>>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        SgxTrustedTlsChannel::<U, V>::new(&self.url, &self.config)
    }

    pub fn config(self, config: SgxTrustedTlsClientConfig) -> Self {
        Self {
            url: self.url,
            config,
        }
    }
}
