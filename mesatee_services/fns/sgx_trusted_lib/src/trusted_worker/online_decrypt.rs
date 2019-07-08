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

use crate::trait_defs::{WorkerHelper, WorkerInput};
use mesatee_core::{Error, ErrorKind, Result};
use ring::aead;
use ring::aead::Aad;
use ring::aead::Nonce;
use serde_derive::Deserialize;
use serde_json;
use std::prelude::v1::*;

// INPUT from file-id
#[derive(Deserialize)]
pub struct AEADKeyConfig {
    pub key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub ad: Vec<u8>,
}
// INPUT: encrypted bytes encoded with base64
// OUTPUT: decypted bytes encoded with base64

pub fn decrypt_data(
    mut data: Vec<u8>,
    aes_key: &[u8],
    aes_nonce: &[u8],
    aes_ad: &[u8],
) -> Result<Vec<u8>> {
    let aead_alg = &aead::AES_256_GCM;
    let o_key = aead::OpeningKey::new(aead_alg, aes_key)
        .map_err(|_| mesatee_core::Error::from(mesatee_core::ErrorKind::CryptoError))?;
    let nonce = Nonce::try_assume_unique_for_key(aes_nonce)
        .map_err(|_| mesatee_core::Error::from(mesatee_core::ErrorKind::CryptoError))?;
    let ad = Aad::from(aes_ad);
    let result = aead::open_in_place(&o_key, nonce, ad, 0, &mut data[..]);
    let decrypted_buffer =
        result.map_err(|_| mesatee_core::Error::from(mesatee_core::ErrorKind::CryptoError))?;
    Ok(decrypted_buffer.to_vec())
}

pub(crate) fn decrypt(helper: &mut WorkerHelper, input: WorkerInput) -> Result<String> {
    let file_id = match input.input_files.get(0) {
        Some(value) => value,
        None => return Err(Error::from(ErrorKind::MissingValue)),
    };
    let key_bytes = helper.read_file(&file_id)?;
    let key_config: AEADKeyConfig = serde_json::from_slice(&key_bytes)
        .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;

    let encypted_base64 = match input.payload {
        Some(value) => value,
        None => return Err(Error::from(ErrorKind::MissingValue)),
    };
    let encypted_bytes: Vec<u8> =
        base64::decode(&encypted_base64).map_err(|_| Error::from(ErrorKind::InvalidInputError))?;

    let decrypted_bytes = decrypt_data(
        encypted_bytes,
        &key_config.key,
        &key_config.nonce,
        &key_config.ad,
    )?;
    let output_base64 = base64::encode(&decrypted_bytes);
    Ok(output_base64)
}
