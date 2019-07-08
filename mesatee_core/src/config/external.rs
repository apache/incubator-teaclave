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
use super::ServiceConfig;
use super::{InboundDesc, OutboundDesc, TargetDesc};
use mesatee_config::MESATEE_CONFIG;

pub struct External;
impl External {
    pub fn tms() -> ServiceConfig {
        ServiceConfig::new(
            MESATEE_CONFIG.tms_external_listen_addr,
            MESATEE_CONFIG.tms_external_port,
            InboundDesc::External,
        )
    }

    pub fn fns() -> ServiceConfig {
        ServiceConfig::new(
            MESATEE_CONFIG.fns_external_listen_addr,
            MESATEE_CONFIG.fns_external_port,
            InboundDesc::External,
        )
    }

    pub fn tdfs() -> ServiceConfig {
        ServiceConfig::new(
            MESATEE_CONFIG.tdfs_external_listen_addr,
            MESATEE_CONFIG.tdfs_external_port,
            InboundDesc::External,
        )
    }

    pub fn target_fns() -> TargetDesc {
        TargetDesc::new(
            MESATEE_CONFIG.fns_external_connect_addr,
            MESATEE_CONFIG.fns_external_port,
            OutboundDesc::Sgx(get_trusted_enclave_attr(vec!["fns"])),
        )
    }
}
