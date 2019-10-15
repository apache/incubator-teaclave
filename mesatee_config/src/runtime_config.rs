// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use lazy_static::lazy_static;
use serde_derive::Deserialize;

use std::collections::BTreeMap;
use std::error;
use std::io::{BufReader, Error, ErrorKind, Result};
use std::path::{Path, PathBuf};

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;
#[cfg(feature = "mesalock_sgx")]
use std::string::String;

#[cfg(not(feature = "mesalock_sgx"))]
use std::fs::File;
#[cfg(feature = "mesalock_sgx")]
use std::untrusted::fs::File;

#[cfg(feature = "mesalock_sgx")]
use std::untrusted::path::PathEx;

use std::env;
use std::io::Read;
use std::net::IpAddr;

#[derive(Debug, Deserialize)]
pub struct MesateeConfigToml {
    pub api_endpoints: BTreeMap<String, ApiEndpoint>,
    pub internal_endpoints: BTreeMap<String, InternalEndpoint>,
    pub ias_client_config: BTreeMap<String, PathValue>,
    pub audited_enclave_config: BTreeMap<String, PathValue>,
}

#[derive(Debug, Deserialize)]
pub struct ApiEndpoint {
    pub listen_ip: IpAddr,
    // `connect_ip` is a required field for FNS only
    pub connect_ip: Option<IpAddr>,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct InternalEndpoint {
    pub listen_ip: IpAddr,
    pub connect_ip: IpAddr,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct PathValue {
    pub path: PathBuf,
}

impl MesateeConfigToml {
    fn is_valid(&self) -> Result<()> {
        let api_endpoints = &self.api_endpoints;
        if !api_endpoints.contains_key("tms") {
            return Err(err("[api_endpoint]: missing `tms`"));
        }
        if !api_endpoints.contains_key("fns") {
            return Err(err("[api_endpoint]: missing `fns`"));
        }
        if !api_endpoints.contains_key("tdfs") {
            return Err(err("[api_endpoint]: missing `tdfs`"));
        }
        if api_endpoints["fns"].connect_ip.is_none() {
            return Err(err("[api_endpoint]: `fns` missing `connect_ip`"));
        }

        let internal_endpoints = &self.internal_endpoints;
        if !internal_endpoints.contains_key("tms") {
            return Err(err("[api_endpoint]: missing `tms`"));
        }
        if !internal_endpoints.contains_key("tdfs") {
            return Err(err("[api_endpoint]: missing `tdfs`"));
        }
        if !internal_endpoints.contains_key("kms") {
            return Err(err("[api_endpoint]: missing `kms`"));
        }
        if !internal_endpoints.contains_key("acs") {
            return Err(err("[api_endpoint]: missing `acs`"));
        }

        let audited_enclave_config = &self.audited_enclave_config;
        if !audited_enclave_config.contains_key("enclave_info") {
            return Err(err("[ias_client_config]: missing `enclave_info`"));
        }
        if !audited_enclave_config.contains_key("signature_a") {
            return Err(err("[ias_client_config]: missing `signature_a`"));
        }
        if !audited_enclave_config.contains_key("signature_b") {
            return Err(err("[ias_client_config]: missing `signature_b`"));
        }
        if !audited_enclave_config.contains_key("signature_c") {
            return Err(err("[ias_client_config]: missing `signature_c`"));
        }

        Ok(())
    }
}

const MESATEE_CFG_DIR_ENV: &str = "MESATEE_CFG_DIR";

lazy_static! {
    pub static ref MESATEE_CONFIG_TOML: MesateeConfigToml = {
        match load_mesatee_runtime_config_from_env() {
            Ok(cfg) => cfg,
            Err(e) => panic!("{:?}", e),
        }
    };
}

#[inline]
fn get_mesatee_cfg_dir() -> String {
    env::var(&MESATEE_CFG_DIR_ENV).expect("Please set $MESATEE_CFG_DIR")
}

#[inline]
fn load_mesatee_runtime_config_from_env() -> Result<MesateeConfigToml> {
    let mesatee_cfg_dir = get_mesatee_cfg_dir();
    let cfg_dir = Path::new(&mesatee_cfg_dir);
    if !cfg_dir.exists() || !cfg_dir.is_dir() {
        return Err(err("$MESATEE_CFG_DIR does not exist or is not a directory"));
    }
    let cfg_file = cfg_dir.to_path_buf().join("config.toml");
    if !cfg_file.exists() || !cfg_file.is_file() {
        return Err(err("config.toml does not exist or is not a file"));
    }

    let f = File::open(cfg_file)?;
    let mut f = BufReader::new(f);
    let mut cfg_dat = String::new();
    let _ = f.read_to_string(&mut cfg_dat)?;

    let config: MesateeConfigToml = toml::from_str(&cfg_dat).map_err(err)?;
    config.is_valid()?;
    Ok(config)
}

pub struct MesateeConfig {
    pub tms_external_listen_addr: IpAddr,
    pub tms_internal_listen_addr: IpAddr,
    pub tms_internal_connect_addr: IpAddr,
    pub tms_external_port: u16,
    pub tms_internal_port: u16,

    pub tdfs_external_listen_addr: IpAddr,
    pub tdfs_internal_listen_addr: IpAddr,
    pub tdfs_internal_connect_addr: IpAddr,
    pub tdfs_external_port: u16,
    pub tdfs_internal_port: u16,

    pub kms_internal_listen_addr: IpAddr,
    pub kms_internal_connect_addr: IpAddr,
    pub kms_internal_port: u16,

    pub acs_internal_listen_addr: IpAddr,
    pub acs_internal_connect_addr: IpAddr,
    pub acs_internal_port: u16,

    pub fns_external_listen_addr: IpAddr,
    pub fns_external_connect_addr: IpAddr, // for TMS to return to users
    pub fns_external_port: u16,

    pub ias_client_spid_path: PathBuf,
    pub ias_client_key_path: PathBuf,

    pub audited_enclave_info_path: PathBuf,
    pub auditor_a_signature_path: PathBuf,
    pub auditor_b_signature_path: PathBuf,
    pub auditor_c_signature_path: PathBuf,
}

lazy_static! {
    pub static ref MESATEE_CONFIG: MesateeConfig = {
        let mesatee_cfg_dir = get_mesatee_cfg_dir();

        let audited_enclave_info_path = Path::new(&mesatee_cfg_dir)
            .to_path_buf()
            .join(&MESATEE_CONFIG_TOML.audited_enclave_config["enclave_info"].path);
        let auditor_a_signature_path = Path::new(&mesatee_cfg_dir)
            .to_path_buf()
            .join(&MESATEE_CONFIG_TOML.audited_enclave_config["signature_a"].path);
        let auditor_b_signature_path = Path::new(&mesatee_cfg_dir)
            .to_path_buf()
            .join(&MESATEE_CONFIG_TOML.audited_enclave_config["signature_b"].path);
        let auditor_c_signature_path = Path::new(&mesatee_cfg_dir)
            .to_path_buf()
            .join(&MESATEE_CONFIG_TOML.audited_enclave_config["signature_c"].path);

        let ias_client_spid_path = Path::new(&mesatee_cfg_dir)
            .to_path_buf()
            .join(&MESATEE_CONFIG_TOML.ias_client_config["spid"].path);

        let ias_client_key_path = Path::new(&mesatee_cfg_dir)
            .to_path_buf()
            .join(&MESATEE_CONFIG_TOML.ias_client_config["key"].path);

        MesateeConfig {
            tms_external_listen_addr: MESATEE_CONFIG_TOML.api_endpoints["tms"].listen_ip,
            tms_internal_listen_addr: MESATEE_CONFIG_TOML.internal_endpoints["tms"].listen_ip,
            tms_internal_connect_addr: MESATEE_CONFIG_TOML.internal_endpoints["tms"].connect_ip,
            tms_external_port: MESATEE_CONFIG_TOML.api_endpoints["tms"].port,
            tms_internal_port: MESATEE_CONFIG_TOML.internal_endpoints["tms"].port,

            tdfs_external_listen_addr: MESATEE_CONFIG_TOML.api_endpoints["tdfs"].listen_ip,
            tdfs_internal_listen_addr: MESATEE_CONFIG_TOML.internal_endpoints["tdfs"].listen_ip,
            tdfs_internal_connect_addr: MESATEE_CONFIG_TOML.internal_endpoints["tdfs"].connect_ip,
            tdfs_external_port: MESATEE_CONFIG_TOML.api_endpoints["tdfs"].port,
            tdfs_internal_port: MESATEE_CONFIG_TOML.internal_endpoints["tdfs"].port,

            kms_internal_listen_addr: MESATEE_CONFIG_TOML.internal_endpoints["kms"].listen_ip,
            kms_internal_connect_addr: MESATEE_CONFIG_TOML.internal_endpoints["kms"].connect_ip,
            kms_internal_port: MESATEE_CONFIG_TOML.internal_endpoints["kms"].port,

            acs_internal_listen_addr: MESATEE_CONFIG_TOML.internal_endpoints["acs"].listen_ip,
            acs_internal_connect_addr: MESATEE_CONFIG_TOML.internal_endpoints["acs"].connect_ip,
            acs_internal_port: MESATEE_CONFIG_TOML.internal_endpoints["acs"].port,

            fns_external_listen_addr: MESATEE_CONFIG_TOML.api_endpoints["fns"].listen_ip,
            // `connect_ip` is a required field for FNS in [api_endpoints];
            // we can unwrap() safely because we have checked it in MesateeConfigToml::is_valid().
            fns_external_connect_addr: MESATEE_CONFIG_TOML.api_endpoints["fns"].connect_ip.unwrap(),
            fns_external_port: MESATEE_CONFIG_TOML.api_endpoints["fns"].port,

            ias_client_spid_path: ias_client_spid_path,
            ias_client_key_path: ias_client_key_path,

            audited_enclave_info_path: audited_enclave_info_path,
            auditor_a_signature_path: auditor_a_signature_path,
            auditor_b_signature_path: auditor_b_signature_path,
            auditor_c_signature_path: auditor_c_signature_path,
        }
    };
}

#[inline(always)]
pub fn err<E>(e: E) -> Error
where
    E: Into<Box<dyn error::Error + Send + Sync>>,
{
    Error::new(ErrorKind::Other, e)
}
