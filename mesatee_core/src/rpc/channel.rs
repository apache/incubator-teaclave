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

// Use sgx_tstd to replace Rust's default std
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::rpc::sgx;
use crate::rpc::RpcClient;
use crate::Result;
use net2::TcpBuilder;
use serde::{de::DeserializeOwned, Serialize};

use teaclave_attestation::verifier::EnclaveAttr;
use teaclave_attestation::verifier::SgxQuoteVerifier;

pub struct SgxTrustedChannel<U: Serialize, V: DeserializeOwned> {
    client: sgx::PipeClient<U, V>,
}

impl<U, V> SgxTrustedChannel<U, V>
where
    U: Serialize,
    V: DeserializeOwned,
{
    pub fn new(
        addr: std::net::SocketAddr,
        enclave_attr: EnclaveAttr,
    ) -> Result<SgxTrustedChannel<U, V>> {
        let tcp_builder = TcpBuilder::new_v4()?;
        tcp_builder.reuse_address(true)?;
        let stream = tcp_builder.connect(addr)?;
        stream.set_nodelay(true)?;

        let config = sgx::PipeClientConfig {
            tcp: stream,
            hostname: webpki::DNSNameRef::try_from_ascii_str(
                format!("{}-{}", "localhost", addr.port()).as_ref(),
            )
            .unwrap()
            .to_owned(),
            server_verifier: SgxQuoteVerifier::new(enclave_attr),
        };
        let client = sgx::PipeClient::<U, V>::open(config)?;

        Ok(SgxTrustedChannel { client })
    }

    pub fn invoke(&mut self, input: U) -> Result<V> {
        self.client.invoke(input)
    }
}
