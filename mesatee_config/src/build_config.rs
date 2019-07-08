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

pub struct MesateeSecurityConstants {
    pub mr_signer: &'static [u8],
    pub root_ca_bin: &'static [u8],
    pub ias_report_ca: &'static [u8],

    pub client_cert: &'static [u8],
    pub client_pkcs8_key: &'static [u8],

    pub audited_enclave_pubkey_a: &'static [u8],
    pub audited_enclave_pubkey_b: &'static [u8],
    pub audited_enclave_pubkey_c: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/gen_build_config.rs"));
