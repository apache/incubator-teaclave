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
use ring::{rand, signature};
use std::prelude::v1::*;
use std::vec;

pub(crate) fn sign(helper: &mut WorkerHelper, input: WorkerInput) -> Result<String> {
    let file_id = match input.input_files.get(0) {
        Some(value) => value,
        None => return Err(Error::from(ErrorKind::MissingValue)),
    };
    let prv_key_der = helper.read_file(&file_id)?;
    let key_pair = signature::RsaKeyPair::from_der(&prv_key_der)
        .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;

    let payload = match input.payload {
        Some(value) => {
            base64::decode(&value).map_err(|_| Error::from(ErrorKind::InvalidInputError))?
        }
        None => return Err(Error::from(ErrorKind::MissingValue)),
    };

    let mut sig = vec![0; key_pair.public_modulus_len()];
    let rng = rand::SystemRandom::new();
    key_pair
        .sign(&signature::RSA_PKCS1_SHA256, &rng, &payload, &mut sig)
        .map_err(|_| Error::from(ErrorKind::CryptoError))?;

    let output_base64 = base64::encode(&sig);
    Ok(output_base64)
}
