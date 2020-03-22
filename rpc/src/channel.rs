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

use crate::config::SgxTrustedTlsClientConfig;
use crate::transport::{ClientTransport, SgxTrustedTlsTransport};
use crate::Request;
use anyhow::anyhow;
use anyhow::Result;
use http::Uri;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct SgxTrustedTlsChannel<U, V>
where
    U: Serialize + std::fmt::Debug,
    V: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    transport: SgxTrustedTlsTransport<rustls::ClientSession>,
    maker: std::marker::PhantomData<(U, V)>,
}

impl<U, V> SgxTrustedTlsChannel<U, V>
where
    U: Serialize + std::fmt::Debug,
    V: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    pub fn new(
        address: &str,
        client_config: &SgxTrustedTlsClientConfig,
    ) -> Result<SgxTrustedTlsChannel<U, V>> {
        let uri = address.parse::<Uri>()?;
        let hostname = uri.host().ok_or_else(|| anyhow!("Invalid hostname."))?;
        let stream = std::net::TcpStream::connect(address)?;
        let hostname = webpki::DNSNameRef::try_from_ascii_str(hostname)?;
        let session =
            rustls::ClientSession::new(&Arc::new(client_config.client_config.clone()), hostname);
        let tls_stream = rustls::StreamOwned::new(session, stream);
        let transport = SgxTrustedTlsTransport::new(tls_stream);

        Ok(Self {
            transport,
            maker: std::marker::PhantomData::<(U, V)>,
        })
    }

    pub fn invoke(
        &mut self,
        input: Request<U>,
    ) -> teaclave_types::TeaclaveServiceResponseResult<V> {
        self.transport.send(input)
    }
}
