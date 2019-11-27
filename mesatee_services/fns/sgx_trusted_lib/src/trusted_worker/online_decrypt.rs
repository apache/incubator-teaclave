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

use crate::worker::{FunctionType, Worker, WorkerContext};
use mesatee_core::{Error, ErrorKind, Result};
use ring::aead::{self, Aad, BoundKey, Nonce, UnboundKey};
use serde_derive::Deserialize;
use serde_json;
use std::prelude::v1::*;

// INPUT from file-id
#[derive(Deserialize)]
struct AEADKeyConfig {
    pub key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub ad: Vec<u8>,
}
// INPUT: encrypted bytes encoded with base64
// OUTPUT: decypted bytes encoded with base64

pub struct OnlineDecryptWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<OnlineDecryptWorkerInput>,
}

struct OnlineDecryptWorkerInput {
    pub encrypted_bytes: Vec<u8>,
    pub aead_file_id: String,
}

impl OnlineDecryptWorker {
    pub fn new() -> Self {
        OnlineDecryptWorker {
            worker_id: 0,
            func_name: "decrypt".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}

impl Worker for OnlineDecryptWorker {
    fn function_name(&self) -> &str {
        self.func_name.as_str()
    }
    fn function_type(&self) -> FunctionType {
        self.func_type
    }
    fn set_id(&mut self, worker_id: u32) {
        self.worker_id = worker_id;
    }
    fn id(&self) -> u32 {
        self.worker_id
    }
    fn prepare_input(
        &mut self,
        dynamic_input: Option<String>,
        file_ids: Vec<String>,
    ) -> Result<()> {
        let aead_file_id = match file_ids.get(0) {
            Some(value) => value.to_string(),
            None => return Err(Error::from(ErrorKind::InvalidInputError)),
        };
        let encrypted_base64 = match dynamic_input {
            Some(value) => value,
            None => return Err(Error::from(ErrorKind::InvalidInputError)),
        };
        let encrypted_bytes: Vec<u8> = base64::decode(&encrypted_base64)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
        self.input = Some(OnlineDecryptWorkerInput {
            encrypted_bytes,
            aead_file_id,
        });
        Ok(())
    }

    fn execute(&mut self, context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        let key_bytes = context.read_file(&input.aead_file_id)?;
        let key_config: AEADKeyConfig = serde_json::from_slice(&key_bytes)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;

        let decrypted_bytes = decrypt_data(
            input.encrypted_bytes,
            &key_config.key,
            &key_config.nonce,
            &key_config.ad,
        )?;
        let output_base64 = base64::encode(&decrypted_bytes);
        Ok(output_base64)
    }
}

struct OneNonceSequence(Option<aead::Nonce>);

impl OneNonceSequence {
    /// Constructs the sequence allowing `advance()` to be called
    /// `allowed_invocations` times.
    fn new(nonce: aead::Nonce) -> Self {
        Self(Some(nonce))
    }
}

impl aead::NonceSequence for OneNonceSequence {
    fn advance(&mut self) -> core::result::Result<aead::Nonce, ring::error::Unspecified> {
        self.0.take().ok_or(ring::error::Unspecified)
    }
}

fn decrypt_data(
    mut data: Vec<u8>,
    aes_key: &[u8],
    aes_nonce: &[u8],
    aes_ad: &[u8],
) -> Result<Vec<u8>> {
    let aead_alg = &aead::AES_256_GCM;
    let ub = UnboundKey::new(aead_alg, aes_key).map_err(|_| Error::from(ErrorKind::CryptoError))?;
    let nonce = Nonce::try_assume_unique_for_key(aes_nonce)
        .map_err(|_| mesatee_core::Error::from(mesatee_core::ErrorKind::CryptoError))?;
    let filesequence = OneNonceSequence::new(nonce);
    let mut o_key = aead::OpeningKey::new(ub, filesequence);
    let ad = Aad::from(aes_ad);
    let result = o_key.open_in_place(ad, &mut data[..]);
    let decrypted_buffer =
        result.map_err(|_| mesatee_core::Error::from(mesatee_core::ErrorKind::CryptoError))?;
    Ok(decrypted_buffer.to_vec())
}
