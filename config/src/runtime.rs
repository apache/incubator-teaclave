use crate::ConfigSource;
#[cfg(not(feature = "mesalock_sgx"))]
use std::fs;
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;
#[cfg(feature = "mesalock_sgx")]
use std::untrusted::fs;

use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use std::string::String;
use std::vec::Vec;
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub api_endpoints: ApiEndpointsConfig,
    pub internal_endpoints: InternalEndpointsConfig,
    pub audit: AuditConfig,
    pub ias: Option<IasConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiEndpointsConfig {
    pub frontend: ApiEndpoint,
    pub authentication: ApiEndpoint,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalEndpointsConfig {
    pub dbs: InternalEndpoint,
    pub execution: InternalEndpoint,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub listen_address: SocketAddr,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalEndpoint {
    pub listen_address: SocketAddr,
    pub advertised_address: SocketAddr,
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
pub struct IasConfig {
    pub ias_spid: String,
    pub ias_key: String,
}

impl RuntimeConfig {
    pub fn from_toml<T: AsRef<Path>>(path: T) -> Option<Self> {
        let contents = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                error!("Something went wrong when reading the runtime config file.");
                return None;
            }
        };
        let mut config: RuntimeConfig = match toml::from_str(&contents) {
            Ok(c) => c,
            Err(_) => {
                error!("Cannot parse the runtime config file.");
                return None;
            }
        };

        config.audit.enclave_info_bytes = match &config.audit.enclave_info_source {
            ConfigSource::Path(ref enclave_info_path) => {
                let content = fs::read(enclave_info_path).unwrap_or_else(|_| {
                    panic!("Cannot find enclave info at {:?}.", enclave_info_path)
                });
                Some(content)
            }
        };

        let mut signatures: Vec<Vec<u8>> = vec![];
        for source in &config.audit.auditor_signatures_source {
            let signature = match source {
                ConfigSource::Path(ref path) => fs::read(path)
                    .unwrap_or_else(|_| panic!("Cannot find signature file {:?}.", path)),
            };
            signatures.push(signature);
        }
        config.audit.auditor_signatures_bytes = Some(signatures);

        if env::var("IAS_SPID").is_ok() && env::var("IAS_KEY").is_ok() {
            let ias_spid = env::var("IAS_SPID").unwrap();
            let ias_key = env::var("IAS_KEY").unwrap();
            config.ias = Some(IasConfig { ias_spid, ias_key });
        }

        if cfg!(sgx_sim) && config.ias.is_none() {
            config.ias = Some(IasConfig {
                ias_spid: "SGX_SIMULATION_MODE_IAS_SPID_123".to_string(),
                ias_key: "SGX_SIMULATION_MODE_IAS_KEY_1234".to_string(),
            });
        }

        if config.ias.is_none()
            || config.ias.as_ref().unwrap().ias_spid.len() != 32
            || config.ias.as_ref().unwrap().ias_key.len() != 32
        {
            error!("Cannot find IAS SPID/key or format error.");
            return None;
        }

        Some(config)
    }
}
