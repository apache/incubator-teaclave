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

use super::get_trusted_enclave_attr;
use super::InboundDesc;
use super::OutboundDesc;
use super::ServiceConfig;
use super::TargetDesc;
use mesatee_config::MESATEE_CONFIG;

pub struct Internal;
impl Internal {
    pub fn tms() -> ServiceConfig {
        ServiceConfig::new(
            MESATEE_CONFIG.tms_internal_listen_addr,
            MESATEE_CONFIG.tms_internal_port,
            InboundDesc::Sgx(get_trusted_enclave_attr(vec!["fns"])),
        )
    }

    pub fn kms() -> ServiceConfig {
        ServiceConfig::new(
            MESATEE_CONFIG.kms_internal_listen_addr,
            MESATEE_CONFIG.kms_internal_port,
            InboundDesc::Sgx(get_trusted_enclave_attr(vec!["fns", "tdfs"])),
        )
    }

    pub fn tdfs() -> ServiceConfig {
        ServiceConfig::new(
            MESATEE_CONFIG.tdfs_internal_listen_addr,
            MESATEE_CONFIG.tdfs_internal_port,
            InboundDesc::Sgx(get_trusted_enclave_attr(vec!["fns", "tms"])),
        )
    }

    pub fn acs() -> ServiceConfig {
        ServiceConfig::new(
            MESATEE_CONFIG.acs_internal_listen_addr,
            MESATEE_CONFIG.acs_internal_port,
            InboundDesc::Sgx(get_trusted_enclave_attr(vec!["kms", "tms", "tdfs"])),
        )
    }

    pub fn target_tms() -> TargetDesc {
        TargetDesc::new(
            MESATEE_CONFIG.tms_internal_connect_addr,
            MESATEE_CONFIG.tms_internal_port,
            OutboundDesc::Sgx(get_trusted_enclave_attr(vec!["tms"])),
        )
    }

    pub fn target_kms() -> TargetDesc {
        TargetDesc::new(
            MESATEE_CONFIG.kms_internal_connect_addr,
            MESATEE_CONFIG.kms_internal_port,
            OutboundDesc::Sgx(get_trusted_enclave_attr(vec!["kms"])),
        )
    }

    pub fn target_tdfs() -> TargetDesc {
        TargetDesc::new(
            MESATEE_CONFIG.tdfs_internal_connect_addr,
            MESATEE_CONFIG.tdfs_internal_port,
            OutboundDesc::Sgx(get_trusted_enclave_attr(vec!["tdfs"])),
        )
    }

    pub fn target_acs() -> TargetDesc {
        TargetDesc::new(
            MESATEE_CONFIG.acs_internal_connect_addr,
            MESATEE_CONFIG.acs_internal_port,
            OutboundDesc::Sgx(get_trusted_enclave_attr(vec!["acs"])),
        )
    }
}
