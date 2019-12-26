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

//! rpc support for MesaTEE

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::{Error, ErrorKind, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, Write};

mod sendrecv;
use crate::rpc::sendrecv::*;

// RpcServer takes three generic type and provides two functions:
// `fn start` to bind the service on the given config
// `fn serve` is a blocking call contains an event loop in which the handler
// functions are invoked.
// Here U is the type of incoming request. V is the type of outgoing response.
// X is the type of the EnclaveService which is a generic type depends on U
// and V.
pub trait RpcServer<U, V, X>: Read + Write
where
    U: DeserializeOwned + std::fmt::Debug,
    V: Serialize + std::fmt::Debug,
    X: EnclaveService<U, V>,
    Self: Sized,
{
    type Config;

    fn start(config: &Self::Config) -> Result<Self>;

    // This call would block -- contains main loop
    // Returns error on socket close or any exceptions.
    // The `loop` here is used for multi-round communications:
    // Assuming that we have the following codes on the client side
    // ```
    // let _ = conn.invoke(first_req).unwrap();
    // let _ = conn.invoke(second_req).unwrap();
    // let _ = conn.invoke(third_req).unwrap();
    // ```
    // The `serve` function would loop its body for 3 times.
    fn serve(&mut self, mut x: X) -> Result<()> {
        loop {
            // First receive a payload from client
            let recv_buf: Vec<u8> = receive_vec(self)?;

            // Now we received a payload in recv_buf
            // recv_buf should be a serialized incoming request U
            // The server needs deser it into U first
            let request: U = serde_json::from_slice(&recv_buf)?;
            debug!("SERVER get request: {:?}", request);
            let result: Result<V> = x.handle_invoke(request).map_err(|e| e.into_simple_error());
            debug!("SERVER handle_invoke result: {:?}", result);

            let response = match serde_json::to_vec(&result) {
                Ok(resp) => resp,
                Err(_) => {
                    let r: Result<V> = Err(Error::from(ErrorKind::InternalRPCError));
                    serde_json::to_vec(&r).expect("infallable")
                }
            };
            debug!("SERVER send response {:?}", response);

            // Now the result is stored in ret and we need to sent it back.
            // `ret` is cleared here. Performance is not very good.
            send_vec(self, response)?;
        }
    }
}

pub trait RpcClient<U, V>: Read + Write
where
    Self: Sized,
    U: Serialize,
    V: DeserializeOwned,
{
    type Config: std::marker::Sized;

    fn open(config: Self::Config) -> Result<Self>;

    fn invoke(&mut self, input: U) -> Result<V> {
        let request_payload: Vec<u8> = serde_json::to_vec(&input)?;

        debug!("CLIENT: sending req: {:?}", request_payload);
        send_vec(self, request_payload)?;

        let result_buf: Vec<u8> = receive_vec(self)?;
        debug!("CLIENT: receiving resp: {:?}", result_buf);

        let resp: Result<V> = serde_json::from_slice(&result_buf)?;

        resp
    }
}

// EnclaveService takes two generic type and provides
pub trait EnclaveService<S, T>
where
    S: DeserializeOwned,
    T: Serialize,
{
    fn handle_invoke(&mut self, input: S) -> Result<T>;
}

// With proper `cfg`s we can support SGX client trusted/untrusted at the same
// time.  But the server supports SGX enclave only.

// TODO: update the following two cfgs if we have a full version sgx rpc
// It seems that the current sgx examples are also dependent on mod unix,
// and thus I choose enable this mode either unix and sgx setting.
// Please reivse this later if we have a dedicated rpc for mesalock_sgx.

pub mod channel;
#[cfg(feature = "mesalock_sgx")]
pub mod server;

pub mod sgx;
pub mod unix;
