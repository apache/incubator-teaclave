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

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::data_store::{self, FileMeta, FILE_STORE};
use kms_proto::{self, KMSClient};
use mesatee_core::config;
use mesatee_core::rpc::EnclaveService;
use mesatee_core::{Error, ErrorKind, Result};
use std::marker::PhantomData;
use tdfs_internal_proto::{CreateFileRequest, DFSRequest, DFSResponse, GetFileRequest};
use uuid::Uuid;

pub trait HandleRequest {
    fn handle_request(&self) -> Result<DFSResponse>;
}

impl HandleRequest for CreateFileRequest {
    fn handle_request(&self) -> Result<DFSResponse> {
        let target = config::Internal::target_kms();
        let mut client = KMSClient::new(target)?;
        let req = kms_proto::proto::CreateKeyRequest::new(kms_proto::EncType::Aead);
        let resp = client.create_key(req)?;
        let key_id = resp.get_key_id();
        let key_config = resp.get_key_config()?;
        let key_config = match key_config {
            kms_proto::KeyConfig::Aead(config) => kms_proto::proto::AeadConfig::from(config),
            kms_proto::KeyConfig::ProtectedFs(_config) => unimplemented!(),
        };

        let file_id = Uuid::new_v4().to_string();
        let file_meta = FileMeta {
            user_id: self.user_id.clone(),
            file_name: self.task_id.clone(),
            sha256: self.sha256.clone(),
            file_size: self.file_size,
            key_id: key_id.clone(),
            storage_path: file_id.clone(),
            task_id: Some(self.task_id.clone()),
            allow_policy: self.allow_policy,
            collaborator_list: self.collaborator_list.to_vec(),
        };
        if FILE_STORE.get(&file_id)?.is_some() {
            return Err(Error::from(ErrorKind::UUIDError));
        }
        data_store::add_file(&file_id, &file_meta)?;

        let resp =
            DFSResponse::new_create_file(&file_id, &file_meta.get_access_path(), &key_config);
        Ok(resp)
    }
}

impl HandleRequest for GetFileRequest {
    fn handle_request(&self) -> Result<DFSResponse> {
        let file_id = &self.file_id;
        let file_meta = FILE_STORE
            .get(file_id)?
            .ok_or_else(|| Error::from(ErrorKind::MissingValue))?;
        let access_path = file_meta.get_access_path();
        let file_info = tdfs_internal_proto::FileInfo {
            user_id: file_meta.user_id,
            file_name: file_meta.file_name,
            sha256: file_meta.sha256,
            file_size: file_meta.file_size,
            access_path,
            task_id: file_meta.task_id,
            collaborator_list: file_meta.collaborator_list,
            allow_policy: file_meta.allow_policy,
            key_id: file_meta.key_id,
        };

        let resp = DFSResponse::new_get_file(&file_info);
        Ok(resp)
    }
}

pub struct DFSInternalEnclave<S, T> {
    state: i32,
    x: PhantomData<S>,
    y: PhantomData<T>,
}

impl<S, T> Default for DFSInternalEnclave<S, T> {
    fn default() -> Self {
        DFSInternalEnclave {
            state: 0,
            x: PhantomData::<S>,
            y: PhantomData::<T>,
        }
    }
}

impl EnclaveService<DFSRequest, DFSResponse> for DFSInternalEnclave<DFSRequest, DFSResponse> {
    fn handle_invoke(&mut self, input: DFSRequest) -> Result<DFSResponse> {
        trace!("handle_invoke invoked!");
        trace!("incoming payload = {:?}", input);
        self.state += 1;
        let response = match input {
            DFSRequest::Create(req) => req.handle_request()?,
            DFSRequest::Get(req) => req.handle_request()?,
        };
        trace!("{}th round complete!", self.state);
        Ok(response)
    }
}
