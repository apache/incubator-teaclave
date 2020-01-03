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

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use uuid::Uuid;

use kms_proto::proto::{
    CreateKeyRequest, CreateKeyResponse, DeleteKeyRequest, DeleteKeyResponse, GetKeyRequest,
    GetKeyResponse, KMSRequest, KMSResponse, KMSService,
};
use kms_proto::{AEADKeyConfig, EncType, KeyConfig};
use lazy_static::lazy_static;
use mesatee_core::db::Memdb;
use mesatee_core::rpc::EnclaveService;
use mesatee_core::{Error, ErrorKind, Result};
use std::marker::PhantomData;

lazy_static! {
    static ref KEY_STORE: Memdb<String, KeyConfig> = {
        let db = Memdb::<String, KeyConfig>::open().expect("cannot open memdb");
        let fake_record = KeyConfig::Aead(AEADKeyConfig {
            key: [65; 32],
            nonce: [65; 12],
            ad: [65; 5],
        });
        let _ = db.set(&"fake_kms_record".to_string(), &fake_record);
        let _ = db.set(&"fake_kms_record_to_be_deleted".to_string(), &fake_record);
        db
    };
}

pub struct KMSEnclave<S, T> {
    x: PhantomData<(S, T)>,
}

impl<S, T> Default for KMSEnclave<S, T> {
    fn default() -> Self {
        KMSEnclave {
            x: PhantomData::<(S, T)>,
        }
    }
}

impl KMSService for KMSEnclave<KMSRequest, KMSResponse> {
    fn get_key(req: GetKeyRequest) -> mesatee_core::Result<GetKeyResponse> {
        let key_config = KEY_STORE
            .get(&req.key_id)?
            .ok_or_else(|| Error::from(ErrorKind::MissingValue))?;

        Ok(GetKeyResponse::new(&key_config))
    }
    fn del_key(req: DeleteKeyRequest) -> mesatee_core::Result<DeleteKeyResponse> {
        let key_config = KEY_STORE
            .del(&req.key_id)?
            .ok_or_else(|| Error::from(ErrorKind::MissingValue))?;

        Ok(DeleteKeyResponse::new(&key_config))
    }
    fn create_key(req: CreateKeyRequest) -> mesatee_core::Result<CreateKeyResponse> {
        let enc_type = req.get_enc_type()?;
        let config = match enc_type {
            EncType::Aead => KeyConfig::new_aead_config(),
            EncType::ProtectedFs => KeyConfig::new_protected_fs_config(),
        };
        let key_id = Uuid::new_v4().to_string();
        if KEY_STORE.get(&key_id)?.is_some() {
            return Err(Error::from(ErrorKind::UUIDError));
        }
        KEY_STORE.set(&key_id, &config)?;
        Ok(CreateKeyResponse::new(&key_id, &config))
    }
}

impl EnclaveService<KMSRequest, KMSResponse> for KMSEnclave<KMSRequest, KMSResponse> {
    fn handle_invoke(&mut self, input: KMSRequest) -> Result<KMSResponse> {
        trace!("handle_invoke invoked!");
        trace!("incoming payload = {:?}", input);
        self.dispatch(input)
    }
}
