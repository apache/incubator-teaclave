use crate::key;
use crate::IasReport;
use anyhow::Result;
use std::prelude::v1::*;
use std::time::{self, SystemTime};
use std::untrusted::time::SystemTimeEx;

const ATTESTATION_VALIDITY_SECS: u64 = 86400u64;

pub struct RemoteAttestation {
    pub time: SystemTime,
    pub validity: time::Duration,
    pub cert: Vec<u8>,
    pub private_key: Vec<u8>,
}

impl RemoteAttestation {
    pub fn generate_and_endorse(ias_key: &str, ias_spid: &str) -> Result<Self> {
        let key_pair = key::Secp256k1KeyPair::new()?;
        let report = if cfg!(sgx_sim) {
            IasReport::default()
        } else {
            IasReport::new(key_pair.pub_k, ias_key, ias_spid)?
        };

        let cert_extension = serde_json::to_vec(&report)?;

        let issuer = "Teaclave";
        let subject = "CN=Teaclave";
        let cert_der = key_pair.create_cert_with_extension(issuer, subject, &cert_extension);
        let prv_key_der = key_pair.private_key_into_der();

        let time = SystemTime::now();
        let validity = time::Duration::from_secs(ATTESTATION_VALIDITY_SECS);

        Ok(Self {
            time,
            validity,
            cert: cert_der,
            private_key: prv_key_der,
        })
    }
}
