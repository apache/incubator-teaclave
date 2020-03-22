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

use log::debug;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io;
use std::mem::transmute;
use std::prelude::v1::*;
use std::vec::Vec;
use teaclave_types::TeaclaveServiceResponseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("IoError")]
    IoError(#[from] io::Error),
    #[error("SerdeError")]
    SerdeError(#[from] serde_json::error::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<ProtocolError> for TeaclaveServiceResponseError {
    fn from(error: ProtocolError) -> Self {
        match error {
            ProtocolError::IoError(e) => {
                TeaclaveServiceResponseError::ConnectionError(format!("{}", e))
            }
            ProtocolError::SerdeError(_) => {
                TeaclaveServiceResponseError::InternalError("serde".to_string())
            }
            ProtocolError::Other(_) => {
                TeaclaveServiceResponseError::InternalError("internal".to_string())
            }
        }
    }
}

pub(crate) struct JsonProtocol<'a, T>
where
    T: io::Read + io::Write,
{
    pub transport: &'a mut T,
    max_frame_len: u64,
}

impl<'a, T> JsonProtocol<'a, T>
where
    T: io::Read + io::Write,
{
    pub fn new(transport: &'a mut T) -> JsonProtocol<'a, T> {
        Self {
            transport,
            // Default max frame length is 8MB
            max_frame_len: 8 * 1_024 * 1_024,
        }
    }

    pub fn read_message<V>(&mut self) -> std::result::Result<V, ProtocolError>
    where
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        let mut header = [0u8; 8];

        self.transport.read_exact(&mut header)?;
        let buf_len = u64::from_be(unsafe { transmute::<[u8; 8], u64>(header) });

        if buf_len > self.max_frame_len {
            return Err(ProtocolError::Other(anyhow::anyhow!(
                "Exceed max frame length"
            )));
        }

        let mut recv_buf: Vec<u8> = vec![0u8; buf_len as usize];
        self.transport.read_exact(&mut recv_buf)?;

        debug!("Recv: {}", std::string::String::from_utf8_lossy(&recv_buf));
        let r: V = serde_json::from_slice(&recv_buf)?;

        Ok(r)
    }

    pub fn write_message<U>(&mut self, message: U) -> std::result::Result<(), ProtocolError>
    where
        U: Serialize + std::fmt::Debug,
    {
        let send_buf = serde_json::to_vec(&message)?;

        debug!("Send: {}", std::string::String::from_utf8_lossy(&send_buf));

        let buf_len = send_buf.len() as u64;
        let header = unsafe { transmute::<u64, [u8; 8]>(buf_len.to_be()) };

        self.transport.write(&header)?;
        self.transport.write_all(&send_buf)?;
        self.transport.flush()?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "result")]
#[serde(rename_all = "snake_case")]
pub enum JsonProtocolResult<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> From<std::result::Result<T, E>> for JsonProtocolResult<T, E> {
    fn from(result: std::result::Result<T, E>) -> Self {
        match result {
            Ok(t) => JsonProtocolResult::Ok(t),
            Err(e) => JsonProtocolResult::Err(e),
        }
    }
}

impl<T, E> From<JsonProtocolResult<T, E>> for std::result::Result<T, E> {
    fn from(result: JsonProtocolResult<T, E>) -> Self {
        match result {
            JsonProtocolResult::Ok(t) => Ok(t),
            JsonProtocolResult::Err(e) => Err(e),
        }
    }
}
