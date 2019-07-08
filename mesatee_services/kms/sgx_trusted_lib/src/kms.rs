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

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;
use std::vec;

use uuid::Uuid;

use mesatee_core::db::Memdb;
use mesatee_core::rpc::EnclaveService;
use mesatee_core::{Error, ErrorKind, Result};
use std::marker::PhantomData;

use kms_proto::AEADKeyConfig;
use kms_proto::KMSResponse;
use kms_proto::{CreateKeyRequest, GetKeyRequest, KMSRequest};

use lazy_static::lazy_static;

lazy_static! {
    static ref KEY_STORE: Memdb<String, AEADKeyConfig> = {
        let db = Memdb::<String, AEADKeyConfig>::open().expect("cannot open memdb");
        let fake_record = AEADKeyConfig {
            key: vec![65; 32],
            nonce: vec![65; 12],
            ad: vec![65; 5],
        };
        let _ = db.set(&"fake_kms_record".to_string(), &fake_record);
        db
    };
}

pub trait HandleRequest {
    fn handle_request(&self) -> Result<KMSResponse>;
}

impl HandleRequest for CreateKeyRequest {
    fn handle_request(&self) -> Result<KMSResponse> {
        let key_config = AEADKeyConfig::new()?;
        let key_id = Uuid::new_v4().to_string();
        if KEY_STORE.get(&key_id)?.is_some() {
            return Err(Error::from(ErrorKind::UUIDError));
        }
        KEY_STORE.set(&key_id, &key_config)?;
        let resp = KMSResponse::new_create_key(&key_id, &key_config);
        Ok(resp)
    }
}

impl HandleRequest for GetKeyRequest {
    fn handle_request(&self) -> Result<KMSResponse> {
        let key_config = KEY_STORE
            .get(&self.key_id)?
            .ok_or_else(|| Error::from(ErrorKind::MissingValue))?;

        let resp = KMSResponse::new_get_key(&key_config);
        Ok(resp)
    }
}

pub struct KMSEnclave<S, T> {
    state: i32,
    x: PhantomData<S>,
    y: PhantomData<T>,
}

impl<S, T> Default for KMSEnclave<S, T> {
    fn default() -> Self {
        KMSEnclave {
            state: 0,
            x: PhantomData::<S>,
            y: PhantomData::<T>,
        }
    }
}

impl EnclaveService<KMSRequest, KMSResponse> for KMSEnclave<KMSRequest, KMSResponse> {
    fn handle_invoke(&mut self, input: KMSRequest) -> Result<KMSResponse> {
        trace!("handle_invoke invoked!");
        trace!("incoming payload = {:?}", input);
        self.state += 1;
        let response = match input {
            KMSRequest::Create(req) => req.handle_request()?,
            KMSRequest::Get(req) => req.handle_request()?,
        };
        trace!("{}th round complete!", self.state);
        Ok(response)
    }
}
