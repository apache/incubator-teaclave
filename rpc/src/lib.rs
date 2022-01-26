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

#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
#[macro_use]
extern crate sgx_tstd as std;

use serde::{Deserialize, Serialize};
use teaclave_types::TeaclaveServiceResponseError;

pub trait TeaclaveService<V, U>
where
    U: Serialize + std::fmt::Debug,
    V: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    fn handle_request(
        &self,
        request: Request<V>,
    ) -> std::result::Result<U, TeaclaveServiceResponseError>;
}

pub mod channel;
pub mod config;
pub mod endpoint;
mod protocol;
mod request;
pub use request::{IntoRequest, Request};
pub use teaclave_rpc_proc_macro::into_request;
#[cfg(feature = "mesalock_sgx")]
pub mod server;
mod transport;
mod utils;
