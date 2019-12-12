// Use sgx_tstd to replace Rust's default std
#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
extern crate sgx_tstd as std;

pub use runtime_config::ConfigSource;

pub mod build_config {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/build_config.rs"));
}

pub mod runtime_config {
    #[cfg(not(feature = "mesalock_sgx"))]
    use std::fs;
    #[cfg(feature = "mesalock_sgx")]
    use std::untrusted::fs;

    use lazy_static::lazy_static;
    use serde_derive::Deserialize;
    use std::env;
    use std::net::SocketAddr;
    use std::path::PathBuf;
    use std::string::String;
    use std::vec::Vec;
    use toml;

    #[derive(Debug, Deserialize)]
    pub struct RuntimeConfig {
        pub api_endpoints: ApiEndpointsConfig,
        pub internal_endpoints: InternalEndpointsConfig,
        pub audit: AuditConfig,
        #[serde(skip_deserializing)]
        pub env: EnvConfig,
    }

    #[derive(Debug, Deserialize)]
    pub struct ApiEndpointsConfig {
        pub tms: EndpointListenConfig,
        pub tdfs: EndpointListenConfig,
        pub fns: EndpointListenAdvertisedConfig,
    }

    #[derive(Debug, Deserialize)]
    pub struct InternalEndpointsConfig {
        pub tms: EndpointListenAdvertisedConfig,
        pub tdfs: EndpointListenAdvertisedConfig,
        pub kms: EndpointListenAdvertisedConfig,
        pub acs: EndpointListenAdvertisedConfig,
    }

    #[derive(Debug, Deserialize)]
    pub struct EndpointListenConfig {
        pub listen_address: SocketAddr,
    }

    #[derive(Debug, Deserialize)]
    pub struct EndpointListenAdvertisedConfig {
        pub listen_address: SocketAddr,
        pub advertised_address: SocketAddr,
    }

    #[derive(Debug, Deserialize)]
    pub struct AuditConfig {
        pub enclave_info: ConfigSource,
        pub auditor_signatures: Vec<ConfigSource>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all(deserialize = "snake_case"))]
    pub enum ConfigSource {
        Path(PathBuf),
    }

    #[derive(Debug, Default)]
    pub struct EnvConfig {
        pub ias_spid: String,
        pub ias_key: String,
    }

    lazy_static! {
        pub static ref RUNTIME_CONFIG: RuntimeConfig = {
            let contents = fs::read_to_string("runtime.config.toml")
                .expect("Something went wrong reading the runtime config file.");
            let mut config: RuntimeConfig = toml::from_str(&contents).unwrap();
            if !cfg!(sgx_sim) {
                let ias_spid =
                    env::var("IAS_SPID").expect("Cannot find IAS_SPID from environment variables.");
                let ias_key =
                    env::var("IAS_KEY").expect("Cannot find IAS_KEY from environment variables.");
                config.env = EnvConfig { ias_spid, ias_key };
            }

            config
        };
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
        println!("{:?}", runtime_config::RUNTIME_CONFIG.env);
    }

    #[test]
    fn test_build_config() {
        println!("{:?}", build_config::BUILD_CONFIG);
    }
}
