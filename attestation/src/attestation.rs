use crate::key;
use crate::AttestationConfig;
use crate::AttestedTlsConfig;
use crate::EndorsedAttestationReport;
use anyhow::Result;
use std::prelude::v1::*;
use std::time::{self, SystemTime};
use std::untrusted::time::SystemTimeEx;

const ATTESTATION_VALIDITY_SECS: u64 = 86400u64;
const CERT_ISSUER: &str = "Teaclave";
const CERT_SUBJECT: &str = "CN=Teaclave";

pub struct RemoteAttestation {
    pub config: AttestationConfig,
}

impl Default for RemoteAttestation {
    fn default() -> Self {
        let config = AttestationConfig::NoAttestation;
        Self { config }
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
        }
    }

    pub fn generate_and_endorse(&self) -> Result<AttestedTlsConfig> {
        let key_pair = key::Secp256k1KeyPair::new()?;
        let report = match &self.config {
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

        Ok(AttestedTlsConfig {
            cert: cert_der,
            private_key: prv_key_der,
            time,
            validity,
        })
    }
}
