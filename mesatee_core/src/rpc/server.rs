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

use crate::rpc::sgx;
use crate::rpc::sgx::EnclaveAttr;
use crate::rpc::EnclaveService;
use crate::rpc::RpcServer;
use crate::Result;
use serde::{de::DeserializeOwned, Serialize};
use sgx_types::c_int;

pub struct SgxTrustedServer<U, V, X>
where
    U: DeserializeOwned + std::fmt::Debug,
    V: Serialize + std::fmt::Debug,
    X: EnclaveService<U, V>,
{
    config: sgx::PipeConfig,
    service: X,
    marker: std::marker::PhantomData<(U, V)>,
}

impl<U, V, X> SgxTrustedServer<U, V, X>
where
    U: DeserializeOwned + std::fmt::Debug,
    V: Serialize + std::fmt::Debug,
    X: EnclaveService<U, V>,
{
    pub fn new(service: X, fd: c_int, client_attr: Option<EnclaveAttr>) -> Result<Self> {
        let config = sgx::PipeConfig { fd, client_attr };
        Ok(Self {
            config,
            service,
            marker: std::marker::PhantomData,
        })
    }

    pub fn start(self) -> Result<()> {
        let mut server = sgx::Pipe::start(&self.config)?;
        server.serve(self.service)
    }
}
