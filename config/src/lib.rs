// Use sgx_tstd to replace Rust's default std
#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
#[macro_use]
extern crate sgx_tstd as std;
#[macro_use]
extern crate log;

pub use runtime_config::ConfigSource;

pub mod build_config {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/build_config.rs"));
}

pub mod runtime_config {
    #[cfg(not(feature = "mesalock_sgx"))]
    use std::fs;
    #[cfg(feature = "mesalock_sgx")]
    use std::prelude::v1::*;
    #[cfg(feature = "mesalock_sgx")]
    use std::untrusted::fs;

    use serde_derive::Deserialize;
    use serde_derive::Serialize;
    use std::env;
    use std::net::SocketAddr;
    use std::path::Path;
    use std::path::PathBuf;
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
        pub frontend: EndpointListenConfig,
        pub authentication: EndpointListenConfig,
        pub tms: EndpointListenConfig,
        pub tdfs: EndpointListenConfig,
        pub fns: EndpointListenAdvertisedConfig,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct InternalEndpointsConfig {
        pub tms: EndpointListenAdvertisedConfig,
        pub tdfs: EndpointListenAdvertisedConfig,
        pub kms: EndpointListenAdvertisedConfig,
        pub acs: EndpointListenAdvertisedConfig,
        pub dbs: EndpointListenAdvertisedConfig,
        pub execution: EndpointListenAdvertisedConfig,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct EndpointListenConfig {
        pub listen_address: SocketAddr,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct EndpointListenAdvertisedConfig {
        pub listen_address: SocketAddr,
        pub advertised_address: SocketAddr,
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
    #[serde(rename_all = "snake_case")]
    pub enum ConfigSource {
        Path(PathBuf),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct IasConfig {
        pub ias_spid: String,
        pub ias_key: String,
    }

    impl RuntimeConfig {
        pub fn from_toml<T: AsRef<Path>>(path: T) -> Option<Self> {
            use std::prelude::v1::*;
            let contents = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => {
                    error!("Something went wrong reading the runtime config file.");
                    return None;
                }
            };
            let mut config: RuntimeConfig = match toml::from_str(&contents) {
                Ok(c) => c,
                Err(_) => {
                    error!("Something went wrong reading the runtime config file.");
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

            if !cfg!(sgx_sim) {
                let ias_spid = match env::var("IAS_SPID") {
                    Ok(e) => e.trim().to_string(),
                    Err(_) => {
                        error!("Cannot find IAS_SPID from environment variables.");
                        return None;
                    }
                };
                let ias_key = match env::var("IAS_KEY") {
                    Ok(e) => e.trim().to_string(),
                    Err(_) => {
                        error!("Cannot find IAS_KEY from environment variables.");
                        return None;
                    }
                };
                if ias_spid.len() != 32 || ias_key.len() != 32 {
                    error!("IAS_SPID or IAS_KEY format error.");
                    return None;
                }

                config.ias = Some(IasConfig { ias_spid, ias_key });
            } else {
                config.ias = Some(IasConfig {
                    ias_spid: "".to_string(),
                    ias_key: "".to_string(),
                });
            }

            Some(config)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_config() {
        println!("{:?}", runtime_config::RUNTIME_CONFIG.api_endpoints);
        println!("{:?}", runtime_config::RUNTIME_CONFIG.internal_endpoints);
        println!("{:?}", runtime_config::RUNTIME_CONFIG.audit);
        println!("{:?}", runtime_config::RUNTIME_CONFIG.ias);
    }

    #[test]
    fn test_build_config() {
        println!("{:?}", build_config::BUILD_CONFIG);
    }
}
