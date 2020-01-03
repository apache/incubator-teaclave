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
use ring::{rand, signature};
use std::prelude::v1::*;
use std::vec;

pub struct RSASignWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<RSASignWorkerInput>,
}
struct RSASignWorkerInput {
    key_file_id: String,
    content: Vec<u8>,
}

impl RSASignWorker {
    pub fn new() -> Self {
        RSASignWorker {
            worker_id: 0,
            func_name: "rsa_sign".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}

impl Worker for RSASignWorker {
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
        let key_file_id = match file_ids.get(0) {
            Some(value) => value.to_string(),
            None => return Err(Error::from(ErrorKind::InvalidInputError)),
        };
        let content = match dynamic_input {
            Some(value) => {
                base64::decode(&value).map_err(|_| Error::from(ErrorKind::InvalidInputError))?
            }
            None => return Err(Error::from(ErrorKind::InvalidInputError)),
        };
        self.input = Some(RSASignWorkerInput {
            key_file_id,
            content,
        });
        Ok(())
    }

    fn execute(&mut self, context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        let prv_key_der = context.read_file(&input.key_file_id)?;
        let key_pair = signature::RsaKeyPair::from_der(&prv_key_der)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;

        let mut sig = vec![0; key_pair.public_modulus_len()];
        let rng = rand::SystemRandom::new();
        key_pair
            .sign(&signature::RSA_PKCS1_SHA256, &rng, &input.content, &mut sig)
            .map_err(|_| Error::from(ErrorKind::CryptoError))?;

        let output_base64 = base64::encode(&sig);
        Ok(output_base64)
    }
}
