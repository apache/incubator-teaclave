use crate::ConfigSource;
#[cfg(not(feature = "mesalock_sgx"))]
use std::fs;
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;
#[cfg(feature = "mesalock_sgx")]
use std::untrusted::fs;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::net;
use std::path::Path;
use std::string::String;
use std::vec::Vec;
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub api_endpoints: ApiEndpointsConfig,
    pub internal_endpoints: InternalEndpointsConfig,
    pub audit: AuditConfig,
    pub attestation: AttestationServiceConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiEndpointsConfig {
    pub frontend: ApiEndpoint,
    pub authentication: ApiEndpoint,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalEndpointsConfig {
    pub access_control: InternalEndpoint,
    pub authentication: InternalEndpoint,
    pub management: InternalEndpoint,
    pub storage: InternalEndpoint,
    pub execution: InternalEndpoint,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub listen_address: net::SocketAddr,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalEndpoint {
    pub listen_address: net::SocketAddr,
    pub advertised_address: String,
    pub inbound_services: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditConfig {
    #[serde(rename(serialize = "enclave_info", deserialize = "enclave_info"))]
    enclave_info_source: ConfigSource,
    #[serde(rename(serialize = "auditor_signatures", deserialize = "auditor_signatures"))]
    auditor_signatures_source: Vec<ConfigSource>,
    pub enclave_info_bytes: Option<Vec<u8>>,
    pub auditor_signatures_bytes: Option<Vec<Vec<u8>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttestationServiceConfig {
    pub algorithm: String,
    pub url: String,
    pub key: String,
    pub spid: String,
}

impl RuntimeConfig {
    pub fn from_toml<T: AsRef<Path>>(path: T) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .context("Something went wrong when reading the runtime config file")?;
        let mut config: RuntimeConfig =
            toml::from_str(&contents).context("Cannot parse the runtime config file")?;

        config.audit.enclave_info_bytes = match &config.audit.enclave_info_source {
            ConfigSource::Path(ref enclave_info_path) => {
                let content = fs::read(enclave_info_path).with_context(|| {
                    format!("Cannot read enclave_info from {:?}", enclave_info_path)
                })?;
                Some(content)
            }
        };

        let mut signatures: Vec<Vec<u8>> = vec![];
        for source in &config.audit.auditor_signatures_source {
            let signature = match source {
                ConfigSource::Path(ref path) => fs::read(path)
                    .with_context(|| format!("Cannot read auditor file from {:?}", path))?,
            };
            signatures.push(signature);
        }
        config.audit.auditor_signatures_bytes = Some(signatures);

        if env::var("AS_ALGO").is_ok()
            && env::var("AS_URL").is_ok()
            && env::var("AS_SPID").is_ok()
            && env::var("AS_KEY").is_ok()
        {
            let algorithm = env::var("AS_ALGO").unwrap();
            let url = env::var("AS_URL").unwrap();
            let spid = env::var("AS_SPID").unwrap();
            let key = env::var("AS_KEY").unwrap();
            config.attestation = AttestationServiceConfig {
                algorithm,
                url,
                key,
                spid,
            };
        }

        validate_config(&config)?;

        Ok(config)
    }
}

fn validate_config(config: &RuntimeConfig) -> Result<()> {
    match config.attestation.algorithm.as_str() {
        "sgx_epid" | "sgx_ecdsa" => (),
        _ => bail!(
            "Invalid attestation algorithm {}",
            config.attestation.algorithm
        ),
    }

    if config.attestation.spid.len() != 32 || config.attestation.key.len() != 32 {
        bail!("Cannot find Attestation Service SPID/key or format error");
    }

    Ok(())
}
