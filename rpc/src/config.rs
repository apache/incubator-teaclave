use anyhow::{anyhow, bail, Result};
use log::debug;
use rustls;
use std::prelude::v1::*;
use std::sync::{Arc, SgxRwLock as RwLock};
use std::time::SystemTime;
use std::untrusted::time::SystemTimeEx;
use teaclave_attestation::report::AttestationReport;
use teaclave_attestation::verifier::AttestationReportVerifier;
use teaclave_attestation::AttestedTlsConfig;
use teaclave_types::EnclaveAttr;

#[derive(Clone)]
pub struct SgxTrustedTlsServerConfig {
    server_config: rustls::ServerConfig,
    attested_tls_config: Option<Arc<RwLock<AttestedTlsConfig>>>,
    time: std::time::SystemTime,
    validity: std::time::Duration,
}

impl Default for SgxTrustedTlsServerConfig {
    fn default() -> Self {
        let client_cert_verifier = rustls::NoClientAuth::new();
        let server_config = rustls::ServerConfig::new(client_cert_verifier);
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
        self.server_config.set_single_cert(cert_chain, key_der)?;

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

    pub fn attestation_report_verifier(
        mut self,
        accepted_enclave_attrs: Vec<EnclaveAttr>,
        root_ca: &[u8],
        verifier: fn(&AttestationReport) -> bool,
    ) -> Result<Self> {
        let verifier = Arc::new(AttestationReportVerifier::new(
            accepted_enclave_attrs,
            root_ca,
            verifier,
        ));

        self.server_config.set_client_certificate_verifier(verifier);
        Ok(Self { ..self })
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

        let mut new_server_config = self.server_config.clone();
        new_server_config.set_single_cert(cert_chain, key_der)?;

        self.server_config = new_server_config;
        self.time = attested_tls_config.time;
        self.validity = attested_tls_config.validity;

        Ok(())
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
    pub fn new() -> Arc<dyn rustls::ServerCertVerifier> {
        Arc::new(NoServerAuth)
    }
}

impl rustls::ServerCertVerifier for NoServerAuth {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        _certs: &[rustls::Certificate],
        _hostname: webpki::DNSNameRef<'_>,
        _ocsp: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(rustls::ServerCertVerified::assertion())
    }
}

impl Default for SgxTrustedTlsClientConfig {
    fn default() -> Self {
        let mut client_config = rustls::ClientConfig::new();

        client_config
            .dangerous()
            .set_certificate_verifier(NoServerAuth::new());
        client_config.versions.clear();
        client_config
            .versions
            .push(rustls::ProtocolVersion::TLSv1_2);

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

    pub fn client_cert(mut self, cert: &[u8], key_der: &[u8]) -> Self {
        let cert_chain = vec![rustls::Certificate(cert.to_vec())];
        let key_der = rustls::PrivateKey(key_der.to_vec());
        self.client_config
            .set_single_client_cert(cert_chain, key_der);

        Self { ..self }
    }

    pub fn from_attested_tls_config(
        attested_tls_config: Arc<RwLock<AttestedTlsConfig>>,
    ) -> Result<Self> {
        let lock = attested_tls_config.clone();
        let tls_config = lock.read().map_err(|_| anyhow!("lock error"))?;
        let mut config = Self::new().client_cert(&tls_config.cert, &tls_config.private_key);
        config.attested_tls_config = Some(attested_tls_config);
        Ok(config)
    }
}
