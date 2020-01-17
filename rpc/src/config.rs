use anyhow::Result;
use rustls;
use std::sync::Arc;
use teaclave_attestation::verifier::AttestationReportVerifier;
use teaclave_attestation::verifier::EnclaveAttr;

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

pub struct SgxTrustedTlsClientConfig {
    pub config: Arc<rustls::ClientConfig>,
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

impl SgxTrustedTlsClientConfig {
    pub fn new_without_verifier() -> Self {
        let mut config = rustls::ClientConfig::new();

        config
            .dangerous()
            .set_certificate_verifier(NoServerAuth::new());
        config.versions.clear();
        config.versions.push(rustls::ProtocolVersion::TLSv1_2);

        Self {
            config: Arc::new(config),
        }
    }

    pub fn new_with_attestation_report_verifier(enclave_attr: EnclaveAttr) -> Self {
        let mut config = rustls::ClientConfig::new();
        let verifier = Arc::new(AttestationReportVerifier::new(enclave_attr));

        config.dangerous().set_certificate_verifier(verifier);
        config.versions.clear();
        config.versions.push(rustls::ProtocolVersion::TLSv1_2);

        Self {
            config: Arc::new(config),
        }
    }
}
