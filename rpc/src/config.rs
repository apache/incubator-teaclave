use anyhow::Result;
use rustls;
use std::sync::Arc;

pub struct SgxTrustedTlsServerConfig {
    pub config: Arc<rustls::ServerConfig>,
}

impl SgxTrustedTlsServerConfig {
    pub fn new_without_verifier(cert: &[u8], key_der: &[u8]) -> Result<Self> {
        let cert_chain = vec![rustls::Certificate(cert.to_vec())];
        let key_der = rustls::PrivateKey(key_der.to_vec());
        let client_cert_verifier = rustls::NoClientAuth::new();
        let mut config = rustls::ServerConfig::new(client_cert_verifier);
        config.set_single_cert(cert_chain, key_der)?;

        Ok(Self {
            config: Arc::new(config),
        })
    }
}
