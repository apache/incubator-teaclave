use anyhow::Result;
use rustls;
use std::prelude::v1::*;
use std::sync::Arc;
use teaclave_attestation::report::AttestationReport;
use teaclave_attestation::verifier::AttestationReportVerifier;
use teaclave_types::EnclaveAttr;

pub struct SgxTrustedTlsServerConfig {
    pub config: rustls::ServerConfig,
}

impl Default for SgxTrustedTlsServerConfig {
    fn default() -> Self {
        let client_cert_verifier = rustls::NoClientAuth::new();
        let config = rustls::ServerConfig::new(client_cert_verifier);

        Self { config }
    }
}

impl SgxTrustedTlsServerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn server_cert(mut self, cert: &[u8], key_der: &[u8]) -> Result<Self> {
        let cert_chain = vec![rustls::Certificate(cert.to_vec())];
        let key_der = rustls::PrivateKey(key_der.to_vec());
        self.config.set_single_cert(cert_chain, key_der)?;

        Ok(Self {
            config: self.config,
        })
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

        self.config.set_client_certificate_verifier(verifier);
        Ok(Self {
            config: self.config,
        })
    }
}

pub struct SgxTrustedTlsClientConfig {
    pub config: rustls::ClientConfig,
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
        let mut config = rustls::ClientConfig::new();

        config
            .dangerous()
            .set_certificate_verifier(NoServerAuth::new());
        config.versions.clear();
        config.versions.push(rustls::ProtocolVersion::TLSv1_2);

        Self { config }
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
        self.config.dangerous().set_certificate_verifier(verifier);

        Self {
            config: self.config,
        }
    }

    pub fn client_cert(mut self, cert: &[u8], key_der: &[u8]) -> Self {
        let cert_chain = vec![rustls::Certificate(cert.to_vec())];
        let key_der = rustls::PrivateKey(key_der.to_vec());
        self.config.set_single_client_cert(cert_chain, key_der);

        Self {
            config: self.config,
        }
    }
}
