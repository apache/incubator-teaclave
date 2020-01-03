use anyhow::Result;
use rustls;
use std::prelude::v1::*;

pub fn new_tls_server_config_without_verifier(
    cert: Vec<u8>,
    key_der: Vec<u8>,
) -> Result<rustls::ServerConfig> {
    let cert_chain = vec![rustls::Certificate(cert)];
    let key_der = rustls::PrivateKey(key_der);
    let client_cert_verifier = rustls::NoClientAuth::new();
    let mut config = rustls::ServerConfig::new(client_cert_verifier);
    config.set_single_cert(cert_chain, key_der)?;

    Ok(config)
}
