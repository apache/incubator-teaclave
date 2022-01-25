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

use crate::protocol;
use crate::Request;
use crate::TeaclaveService;
use anyhow::Result;
use log::warn;
use serde::{Deserialize, Serialize};
use std::prelude::v1::*;

pub(crate) trait ClientTransport {
    fn send<U, V>(
        &mut self,
        request: Request<U>,
    ) -> teaclave_types::TeaclaveServiceResponseResult<V>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug;
}

pub(crate) trait ServerTransport {
    fn serve<U, V, X>(&mut self, service: X) -> Result<()>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
        X: TeaclaveService<V, U>;
}
pub(crate) struct SgxTrustedTlsTransport<S>
where
    S: rustls::Session,
{
    stream: rustls::StreamOwned<S, std::net::TcpStream>,
}

impl<S> SgxTrustedTlsTransport<S>
where
    S: rustls::Session,
{
    pub fn new(stream: rustls::StreamOwned<S, std::net::TcpStream>) -> SgxTrustedTlsTransport<S> {
        SgxTrustedTlsTransport::<S> { stream }
    }
}

impl<S> ClientTransport for SgxTrustedTlsTransport<S>
where
    S: rustls::Session,
{
    fn send<U, V>(
        &mut self,
        request: Request<U>,
    ) -> teaclave_types::TeaclaveServiceResponseResult<V>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        let mut protocol = protocol::JsonProtocol::new(&mut self.stream);
        protocol.write_message(request)?;
        protocol.read_message::<protocol::JsonProtocolResult<
                V,
                teaclave_types::TeaclaveServiceResponseError,
            >>()?
            .into()
    }
}

impl<S> ServerTransport for SgxTrustedTlsTransport<S>
where
    S: rustls::Session,
{
    fn serve<U, V, X>(&mut self, service: X) -> Result<()>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
        X: TeaclaveService<V, U>,
    {
        use crate::protocol::{JsonProtocol, JsonProtocolResult};
        use teaclave_types::TeaclaveServiceResponseError;
        let mut protocol = JsonProtocol::new(&mut self.stream);

        loop {
            let request: Request<V> = match protocol.read_message::<Request<V>>() {
                Ok(r) => r,
                Err(e) => match e {
                    protocol::ProtocolError::IoError(e) => {
                        log::debug!("Connection disconnected: {:?}", e);
                        return Ok(());
                    }
                    _ => {
                        warn!("Connection error: {:?}", e);
                        let response: JsonProtocolResult<U, TeaclaveServiceResponseError> =
                            Err(TeaclaveServiceResponseError::RequestError(
                                "invalid request".to_string(),
                            ))
                            .into();
                        protocol.write_message(response)?;
                        return Ok(());
                    }
                },
            };
            let response: JsonProtocolResult<U, TeaclaveServiceResponseError> =
                service.handle_request(request).into();
            protocol.write_message(response)?;
        }
    }
}
