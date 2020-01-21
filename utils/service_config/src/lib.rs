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

// ip/port is dynamically dispatched for fns client.
// we cannot use the &'static str in this struct.

#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
#[macro_use]
extern crate sgx_tstd as std;

use std::net::SocketAddr;
use std::prelude::v1::*;
use teaclave_attestation;
use teaclave_attestation::verifier::EnclaveAttr;
use teaclave_config::build_config::BUILD_CONFIG;
use teaclave_config::runtime_config;
use teaclave_config::runtime_config::RuntimeConfig;
use teaclave_types::EnclaveInfo;
use teaclave_types::EnclaveMeasurement;

mod external;
mod internal;
pub use external::External;
pub use internal::Internal;

#[derive(Clone)]
pub struct TargetDesc {
    pub addr: SocketAddr,
    pub desc: OutboundDesc,
}

impl TargetDesc {
    pub fn new(addr: SocketAddr, desc: OutboundDesc) -> TargetDesc {
        TargetDesc { addr, desc }
    }
}

#[derive(Clone)]
pub enum InboundDesc {
    Sgx(EnclaveAttr),
    External,
}

#[derive(Clone)]
pub enum OutboundDesc {
    Sgx(EnclaveAttr),
}

impl OutboundDesc {
    pub fn default() -> OutboundDesc {
        OutboundDesc::Sgx(get_trusted_enclave_attr(vec!["fns"]))
    }

    pub fn new(measures: EnclaveMeasurement) -> OutboundDesc {
        OutboundDesc::Sgx(EnclaveAttr {
            measures: vec![measures],
        })
    }
}

pub struct ServiceConfig {
    pub addr: SocketAddr,
    pub inbound_desc: InboundDesc, // Trusted
}

impl ServiceConfig {
    pub fn new(addr: SocketAddr, inbound_desc: InboundDesc) -> ServiceConfig {
        ServiceConfig { addr, inbound_desc }
    }
}

use lazy_static::lazy_static;

fn load_presigned_enclave_info() -> EnclaveInfo {
    let audit = &runtime_config().audit;
    let auditor_signatures_bytes = audit.auditor_signatures_bytes.as_ref().unwrap();
    let enclave_info_bytes = audit.enclave_info_bytes.as_ref().unwrap();
    if auditor_signatures_bytes.len() < BUILD_CONFIG.auditor_public_keys.len() {
        panic!("Number of auditor signatures is not enough for verification.")
    }

    if !EnclaveInfo::verify_enclave_info(
        enclave_info_bytes,
        BUILD_CONFIG.auditor_public_keys,
        auditor_signatures_bytes,
    ) {
        panic!("Failed to verify the signatures of enclave info.");
    }

    EnclaveInfo::load_enclave_info(enclave_info_bytes)
}

lazy_static! {
    static ref RUNTIME_CONFIG: Option<RuntimeConfig> =
        RuntimeConfig::from_toml("runtime.config.toml");
    static ref ENCLAVE_IDENTITIES: EnclaveInfo = load_presigned_enclave_info();
}

pub fn is_runtime_config_initialized() -> bool {
    RUNTIME_CONFIG.is_some()
}

pub fn runtime_config() -> &'static RuntimeConfig {
    RUNTIME_CONFIG
        .as_ref()
        .expect("Invalid runtime config, should gracefully exit during enclave_init!")
}

pub fn get_trusted_enclave_attr(service_names: Vec<&str>) -> EnclaveAttr {
    let measures = service_names
        .iter()
        .map(|name| *ENCLAVE_IDENTITIES.measurements.get(*name).unwrap())
        .collect();
    EnclaveAttr { measures }
}
