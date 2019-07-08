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

// ip/port is dynamically dispatched for fns client.
// we cannot use the &'static str in this struct.

use crate::rpc::sgx::load_presigned_enclave_info;
use crate::rpc::sgx::{EnclaveAttr, SgxMeasure};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::prelude::v1::*;

#[derive(Clone)]
pub struct TargetDesc {
    pub addr: SocketAddr,
    pub desc: OutboundDesc,
}

impl TargetDesc {
    pub fn new(ip: IpAddr, port: u16, desc: OutboundDesc) -> TargetDesc {
        TargetDesc {
            addr: SocketAddr::new(ip, port),
            desc,
        }
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

    pub fn new(measures: (SgxMeasure, SgxMeasure)) -> OutboundDesc {
        OutboundDesc::Sgx(EnclaveAttr {
            measures: vec![measures],
            quote_checker: universal_quote_check,
        })
    }
}

pub struct ServiceConfig {
    pub addr: SocketAddr,
    pub inbound_desc: InboundDesc, // Trusted
}

impl ServiceConfig {
    pub fn new(ip: IpAddr, port: u16, inbound_desc: InboundDesc) -> ServiceConfig {
        ServiceConfig {
            addr: SocketAddr::new(ip, port),
            inbound_desc,
        }
    }
}

use lazy_static::lazy_static;

lazy_static! {
    pub static ref ENCLAVE_IDENTITIES: HashMap<String, (SgxMeasure, SgxMeasure)> =
        load_presigned_enclave_info();
}

// let (test_mr_signer, test_mr_enclave) = ENCLAVE_IDENTITIES.get("functional_test").unwrap();
pub fn get_trusted_enclave_attr(service_names: Vec<&str>) -> EnclaveAttr {
    let measures = service_names
        .iter()
        .map(|name| *ENCLAVE_IDENTITIES.get(&name.to_string()).unwrap())
        .collect();
    EnclaveAttr {
        measures,
        quote_checker: universal_quote_check,
    }
}

use crate::rpc::sgx::auth::*;

pub(crate) fn universal_quote_check(quote: &SgxQuote) -> bool {
    quote.status != SgxQuoteStatus::UnknownBadStatus
}

mod external;
pub use external::External;
mod internal;
pub use internal::Internal;
