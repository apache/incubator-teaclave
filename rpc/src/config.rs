// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use crate::transport::{ClientTlsConfig, ServerTlsConfig};
use anyhow::{anyhow, bail, Result};
use log::debug;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

#[cfg(feature = "mesalock_sgx")]
#[allow(unused_imports)]
use std::untrusted::time::SystemTimeEx;

use teaclave_attestation::report::AttestationReport;
use teaclave_attestation::verifier::AttestationReportVerifier;
use teaclave_attestation::AttestedTlsConfig;
use teaclave_types::EnclaveAttr;

// Yout should set the 'h2' negotiation flag for tonic grpc.
pub const ALPN_H2: &str = "h2";

#[derive(Clone)]
pub struct SgxTrustedTlsServerConfig {
    server_config: rustls::ServerConfig,
    attested_tls_config: Option<Arc<RwLock<AttestedTlsConfig>>>,
    time: std::time::SystemTime,
    validity: std::time::Duration,
}

// Refer to `rustls/src/server/handy.rs` in rustls 0.21.2
// Something which always resolves to the same cert chain.
struct AlwaysResolvesChain(Arc<rustls::sign::CertifiedKey>);

impl AlwaysResolvesChain {
    pub(super) fn new(
        chain: Vec<rustls::Certificate>,
        priv_key: &rustls::PrivateKey,
    ) -> Result<Self, rustls::Error> {
        let key = rustls::sign::any_supported_type(priv_key)
            .map_err(|_| rustls::Error::General("invalid private key".into()))?;
        Ok(Self(Arc::new(rustls::sign::CertifiedKey::new(chain, key))))
    }
}

impl rustls::server::ResolvesServerCert for AlwaysResolvesChain {
    fn resolve(
        &self,
        _client_hello: rustls::server::ClientHello,
    ) -> Option<Arc<rustls::sign::CertifiedKey>> {
        Some(Arc::clone(&self.0))
    }
}

// Refer to `rustls/src/client/handy.rs` in rustls 0.21.2
impl rustls::client::ResolvesClientCert for AlwaysResolvesChain {
    fn resolve(
        &self,
        _acceptable_issuers: &[&[u8]],
        _sigschemes: &[rustls::SignatureScheme],
    ) -> Option<Arc<rustls::sign::CertifiedKey>> {
        Some(Arc::clone(&self.0))
    }

    fn has_certs(&self) -> bool {
        true
    }
}

impl Default for SgxTrustedTlsServerConfig {
    fn default() -> Self {
        let server_config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_cert_resolver(Arc::new(rustls::server::ResolvesServerCertUsingSni::new()));
        let time = SystemTime::now();
        let validity = std::time::Duration::from_secs(u64::max_value());

        Self {
            server_config,
            attested_tls_config: None,
            time,
            validity,
        }
    }
}

impl SgxTrustedTlsServerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn server_cert(mut self, cert: &[u8], key_der: &[u8]) -> Result<Self> {
        let cert_chain = vec![rustls::Certificate(cert.to_vec())];
        let key_der = rustls::PrivateKey(key_der.to_vec());
        let resolver = AlwaysResolvesChain::new(cert_chain, &key_der)?;
        self.server_config.cert_resolver = Arc::new(resolver);

        Ok(Self { ..self })
    }

    pub fn from_attested_tls_config(
        attested_tls_config: Arc<RwLock<AttestedTlsConfig>>,
    ) -> Result<Self> {
        let lock = attested_tls_config.clone();
        let tls_config = lock.read().map_err(|_| anyhow!("lock error"))?;
        let mut config = Self::new().server_cert(&tls_config.cert, &tls_config.private_key)?;
        config.attested_tls_config = Some(attested_tls_config);
        config.time = tls_config.time;
        config.validity = tls_config.validity;
        Ok(config)
    }

    // Disable this function for non-SGX targets.
    #[cfg(any(feature = "mesalock_sgx", feature = "libos"))]
    pub fn attestation_report_verifier(
        self,
        accepted_enclave_attrs: Vec<EnclaveAttr>,
        root_ca: &[u8],
        verifier: fn(&AttestationReport) -> bool,
    ) -> Result<Self> {
        let verifier = Arc::new(AttestationReportVerifier::new(
            accepted_enclave_attrs,
            root_ca,
            verifier,
        ));
        let server_config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(verifier)
            .with_cert_resolver(self.server_config.cert_resolver);

        Ok(Self {
            server_config,
            ..self
        })
    }

    pub fn server_config(&self) -> Arc<rustls::ServerConfig> {
        Arc::new(self.server_config.clone())
    }

    pub fn need_refresh(&self) -> bool {
        let current_time = SystemTime::now();
        let elapsed_time = current_time
            .duration_since(self.time)
            .unwrap_or(self.validity);
        debug!(
            "current_time: {:?}, self.time: {:?}, elapsed time: {:?}, self.validity: {:?}",
            current_time, self.time, elapsed_time, self.validity
        );

        elapsed_time >= self.validity
    }

    pub fn refresh_server_config(&mut self) -> Result<()> {
        let lock = match &self.attested_tls_config {
            Some(config) => config,
            None => bail!("Attestation TLS Config is not set"),
        };
        let attested_tls_config = lock.read().map_err(|_| anyhow!("lock error"))?;
        let cert_chain = vec![rustls::Certificate(attested_tls_config.cert.to_vec())];
        let key_der = rustls::PrivateKey(attested_tls_config.private_key.to_vec());

        let resolver = AlwaysResolvesChain::new(cert_chain, &key_der)?;
        self.server_config.cert_resolver = Arc::new(resolver);

        self.time = attested_tls_config.time;
        self.validity = attested_tls_config.validity;

        Ok(())
    }
}

impl From<SgxTrustedTlsServerConfig> for ServerTlsConfig {
    fn from(config: SgxTrustedTlsServerConfig) -> Self {
        let mut config_service = config.server_config;
        config_service.alpn_protocols = vec![ALPN_H2.as_bytes().to_vec()];
        let mut tls_config = ServerTlsConfig::new();
        let tls_config = tls_config.rustls_server_config(config_service);
        tls_config.to_owned()
    }
}

pub struct SgxTrustedTlsClientConfig {
    pub client_config: rustls::ClientConfig,
    pub attested_tls_config: Option<Arc<RwLock<AttestedTlsConfig>>>,
    pub validity: std::time::Duration,
}

struct NoServerAuth;

impl NoServerAuth {
    // Allow new_ret_no_self, make it consistent with rustls definition of
    // `NoClientAuth::new()`.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Arc<dyn rustls::client::ServerCertVerifier> {
        Arc::new(NoServerAuth)
    }
}

impl rustls::client::ServerCertVerifier for NoServerAuth {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp: &[u8],
        _now: SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

impl Default for SgxTrustedTlsClientConfig {
    fn default() -> Self {
        let client_config = rustls::ClientConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_protocol_versions(&[&rustls::version::TLS12])
            .unwrap()
            .with_custom_certificate_verifier(NoServerAuth::new())
            .with_no_client_auth();

        Self {
            client_config,
            attested_tls_config: None,
            validity: std::time::Duration::default(),
        }
    }
}

impl SgxTrustedTlsClientConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn attestation_report_verifier(
        mut self,
        accepted_enclave_attrs: Vec<EnclaveAttr>,
        root_ca: &[u8],
        verifier: fn(&AttestationReport) -> bool,
    ) -> Self {
        let verifier = Arc::new(AttestationReportVerifier::new(
            accepted_enclave_attrs,
            root_ca,
            verifier,
        ));
        self.client_config
            .dangerous()
            .set_certificate_verifier(verifier);

        Self { ..self }
    }

    pub fn client_cert(mut self, cert: &[u8], key_der: &[u8]) -> Result<Self> {
        let cert_chain = vec![rustls::Certificate(cert.to_vec())];
        let key_der = rustls::PrivateKey(key_der.to_vec());
        let resolver = AlwaysResolvesChain::new(cert_chain, &key_der)?;
        self.client_config.client_auth_cert_resolver = Arc::new(resolver);

        Ok(Self { ..self })
    }

    pub fn from_attested_tls_config(
        attested_tls_config: Arc<RwLock<AttestedTlsConfig>>,
    ) -> Result<Self> {
        let lock = attested_tls_config.clone();
        let tls_config = lock.read().map_err(|_| anyhow!("lock error"))?;
        let mut config = Self::new().client_cert(&tls_config.cert, &tls_config.private_key)?;
        config.attested_tls_config = Some(attested_tls_config);
        Ok(config)
    }
}

impl From<SgxTrustedTlsClientConfig> for ClientTlsConfig {
    fn from(config: SgxTrustedTlsClientConfig) -> Self {
        let mut client_config = config.client_config;
        // Yout must set the 'h2' negotiation flag.
        client_config.alpn_protocols = vec![ALPN_H2.as_bytes().to_vec()];
        ClientTlsConfig::new().rustls_client_config(client_config)
    }
}
