use crate::key;
use crate::AttestationConfig;
use crate::AttestedTlsConfig;
use crate::EndorsedAttestationReport;
use anyhow::{anyhow, Result};
use log::debug;
use std::prelude::v1::*;
use std::sync::{Arc, SgxRwLock as RwLock};
use std::thread;
use std::time::Duration;
use std::time::{self, SystemTime};
use std::untrusted::time::SystemTimeEx;
use teaclave_config::BUILD_CONFIG;

const ATTESTATION_VALIDITY_SECS: u64 = BUILD_CONFIG.attestation_validity_secs;
const CERT_ISSUER: &str = "Teaclave";
const CERT_SUBJECT: &str = "CN=Teaclave";

pub struct RemoteAttestation {
    pub config: AttestationConfig,
    pub attested_tls_config: Option<Arc<RwLock<AttestedTlsConfig>>>,
}

impl Default for RemoteAttestation {
    fn default() -> Self {
        let config = AttestationConfig::NoAttestation;
        Self {
            config,
            attested_tls_config: None,
        }
    }
}

impl RemoteAttestation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn config(mut self, config: AttestationConfig) -> Self {
        self.config = config;
        Self {
            config: self.config,
            attested_tls_config: self.attested_tls_config,
        }
    }

    pub fn generate_and_endorse(self) -> Result<Self> {
        let attested_tls_config = Arc::new(RwLock::new(AttestedTlsConfig::new(&self.config)?));
        let config = self.config.clone();
        let attested_tls_config_ref = attested_tls_config.clone();
        thread::spawn(move || {
            AttestationFreshnessKeeper::new(config, attested_tls_config_ref).start()
        });
        Ok(Self {
            config: self.config,
            attested_tls_config: Some(attested_tls_config),
        })
    }

    pub fn attested_tls_config(&self) -> Option<Arc<RwLock<AttestedTlsConfig>>> {
        self.attested_tls_config.clone()
    }
}

impl AttestedTlsConfig {
    fn new(config: &AttestationConfig) -> Result<AttestedTlsConfig> {
        let key_pair = key::Secp256k1KeyPair::new()?;
        let report = match config {
            AttestationConfig::NoAttestation => EndorsedAttestationReport::default(),
            AttestationConfig::WithAttestation(config) => {
                EndorsedAttestationReport::new(&config, key_pair.pub_k)?
            }
        };

        let cert_extension = serde_json::to_vec(&report)?;
        let cert_der =
            key_pair.create_cert_with_extension(CERT_ISSUER, CERT_SUBJECT, &cert_extension);
        let prv_key_der = key_pair.private_key_into_der();

        let time = SystemTime::now();
        let validity = time::Duration::from_secs(ATTESTATION_VALIDITY_SECS);

        let attested_tls_config = AttestedTlsConfig {
            cert: cert_der,
            private_key: prv_key_der,
            time,
            validity,
        };

        debug!("{:?}", attested_tls_config);

        Ok(attested_tls_config)
    }
}

struct AttestationFreshnessKeeper {
    config: AttestationConfig,
    attested_tls_config: Arc<RwLock<AttestedTlsConfig>>,
}

impl AttestationFreshnessKeeper {
    pub(crate) fn new(
        config: AttestationConfig,
        attested_tls_config: Arc<RwLock<AttestedTlsConfig>>,
    ) -> Self {
        Self {
            config,
            attested_tls_config,
        }
    }

    pub(crate) fn start(&self) {
        debug!("AttestationFreshnessKeeper started");
        loop {
            thread::sleep(Duration::from_secs(ATTESTATION_VALIDITY_SECS));
            match self.refresh() {
                Ok(_) => debug!("Attestation report updated successfully"),
                Err(e) => debug!("Failed to refresh attestation report: {:?}", e),
            }
        }
    }

    fn refresh(&self) -> Result<()> {
        debug!("begin refresh");
        let updated_attested_tls_config = AttestedTlsConfig::new(&self.config)?;
        let lock = self.attested_tls_config.clone();
        let mut config = lock
            .write()
            .map_err(|_| anyhow!("Failed to get write lock"))?;
        *config = updated_attested_tls_config;
        debug!("refresh done");
        Ok(())
    }
}
