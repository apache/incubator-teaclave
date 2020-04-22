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

//! This module provide attestation public APIs in server side.

use std::prelude::v1::*;

use crate::key;
use crate::AttestationConfig;
use crate::AttestedTlsConfig;
use crate::EndorsedAttestationReport;

use std::sync::{Arc, SgxRwLock as RwLock};
use std::thread;
use std::time::{Duration, SystemTime};
use std::untrusted::time::SystemTimeEx;

use anyhow::{anyhow, Result};
use log::debug;
use teaclave_config::build::ATTESTATION_VALIDITY_SECS;

const CERT_ISSUER: &str = "Teaclave";
const CERT_SUBJECT: &str = "CN=Teaclave";

pub struct RemoteAttestation {
    attestation_config: Arc<AttestationConfig>,
    attested_tls_config: Option<Arc<RwLock<AttestedTlsConfig>>>,
}

impl RemoteAttestation {
    /// Construct a `RemoteAttestation` with attestation configuration.
    pub fn new(attestation_config: Arc<AttestationConfig>) -> Self {
        Self {
            attestation_config,
            attested_tls_config: None,
        }
    }

    /// Generate a endorsed attestation report.
    pub fn generate_and_endorse(self) -> Result<Self> {
        let attested_tls_config = Arc::new(RwLock::new(AttestedTlsConfig::new(
            &self.attestation_config,
        )?));
        let attestation_config_ref = self.attestation_config.clone();
        let attested_tls_config_ref = attested_tls_config.clone();
        thread::spawn(move || {
            AttestationFreshnessKeeper::new(attestation_config_ref, attested_tls_config_ref).start()
        });
        Ok(Self {
            attestation_config: self.attestation_config,
            attested_tls_config: Some(attested_tls_config),
        })
    }

    /// Construct a attested TLS config for TLS connection (RPC in Teaclave).
    pub fn attested_tls_config(&self) -> Option<Arc<RwLock<AttestedTlsConfig>>> {
        self.attested_tls_config.clone()
    }
}

impl AttestedTlsConfig {
    fn new(attestation_config: &AttestationConfig) -> Result<AttestedTlsConfig> {
        let key_pair = key::NistP256KeyPair::new()?;
        let report = match attestation_config {
            AttestationConfig::NoAttestation => EndorsedAttestationReport::default(),
            AttestationConfig::WithAttestation(config) => {
                EndorsedAttestationReport::new(&config, key_pair.pub_k())?
            }
        };

        let extension = serde_json::to_vec(&report)?;
        let cert = key_pair.create_cert_with_extension(CERT_ISSUER, CERT_SUBJECT, &extension);
        let private_key = key_pair.private_key_into_der();
        let time = SystemTime::now();
        let validity = Duration::from_secs(ATTESTATION_VALIDITY_SECS);

        let attested_tls_config = AttestedTlsConfig {
            cert,
            private_key,
            time,
            validity,
        };

        debug!("{:?}", attested_tls_config);

        Ok(attested_tls_config)
    }
}

/// To keep attestation report fresh. Refresh current valid report periodically.
struct AttestationFreshnessKeeper {
    attestation_config: Arc<AttestationConfig>,
    attested_tls_config: Arc<RwLock<AttestedTlsConfig>>,
}

impl AttestationFreshnessKeeper {
    pub(crate) fn new(
        attestation_config: Arc<AttestationConfig>,
        attested_tls_config: Arc<RwLock<AttestedTlsConfig>>,
    ) -> Self {
        Self {
            attestation_config,
            attested_tls_config,
        }
    }

    /// Start the fresshness keeper which will periodically refresh it's
    /// `attested_tls_config`.
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

    /// Get updated report form attestation service and create an updated
    /// attested TLS config.
    fn refresh(&self) -> Result<()> {
        debug!("begin refresh");
        let updated_attested_tls_config = AttestedTlsConfig::new(&self.attestation_config)?;
        let lock = self.attested_tls_config.clone();
        let mut config = lock
            .write()
            .map_err(|_| anyhow!("Failed to get write lock"))?;
        *config = updated_attested_tls_config;
        debug!("refresh done");
        Ok(())
    }
}
