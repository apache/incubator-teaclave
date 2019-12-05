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

use crate::file_util;
use core::convert::TryInto;
use kms_proto;
use kms_proto::KMSClient;
use mesatee_core::config::{self, OutboundDesc, TargetDesc};
use mesatee_core::rpc::channel::SgxTrustedChannel;
use mesatee_core::{self, Result};
use std::io::{Read, Write};
use std::untrusted::fs;
use tdfs_internal_proto::{CreateFileResponse, DFSRequest, DFSResponse, GetFileResponse};

pub struct TDFSClient {
    channel: SgxTrustedChannel<DFSRequest, DFSResponse>,
}

impl TDFSClient {
    pub fn new(target: TargetDesc) -> Result<Self> {
        let addr = target.addr;

        let channel = match target.desc {
            OutboundDesc::Sgx(enclave_addr) => {
                SgxTrustedChannel::<DFSRequest, DFSResponse>::new(addr, enclave_addr)?
            }
        };

        Ok(TDFSClient { channel })
    }

    #[allow(clippy::too_many_arguments)]
    fn request_create_file(
        &mut self,
        sha256: &str,
        file_size: u32,
        user_id: &str,
        task_id: &str,
        task_token: &str,
        collaborator_list: &[&str],
        allow_policy: u32,
    ) -> Result<CreateFileResponse> {
        let req = DFSRequest::new_create_file(
            sha256,
            file_size,
            user_id,
            task_id,
            task_token,
            collaborator_list,
            allow_policy,
        );
        let resp = self.channel.invoke(req)?;
        match resp {
            DFSResponse::Create(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::RPCResponseError,
            )),
        }
    }

    fn request_get_file(&mut self, file_id: &str, task_id: &str, task_token: &str) -> Result<GetFileResponse> {
        let req = DFSRequest::new_get_file(file_id, task_id, task_token);
        let resp = self.channel.invoke(req)?;
        match resp {
            DFSResponse::Get(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::RPCResponseError,
            )),
        }
    }

    pub fn save_file(
        &mut self,
        data: &[u8],
        user_id: &str,
        task_id: &str,
        task_token: &str,
        collaborator_list: &[&str],
        allow_policy: u32,
    ) -> Result<String> {
        let data = data.to_vec();
        let sha256 = file_util::cal_hash(&data)?;
        let file_size = data.len() as u32;
        let resp = self.request_create_file(
            &sha256,
            file_size,
            user_id,
            task_id,
            task_token,
            collaborator_list,
            allow_policy,
        )?;
        let file_id = resp.file_id;
        let access_path = resp.access_path;
        let key_config = resp.key_config;
        let encrypted_data =
            file_util::encrypt_data(data, &key_config.key, &key_config.nonce, &key_config.ad)?;
        let mut f = fs::File::create(access_path)?;
        for chunk in encrypted_data.chunks(1024 * 1024) {
            f.write_all(chunk)?;
        }
        Ok(file_id)
    }

    pub fn read_file(&mut self, file_id: &str, task_id: &str, task_token: &str) -> Result<Vec<u8>> {        
        let resp = self.request_get_file(file_id, task_id, task_token)?;
        let file_info = resp.file_info;

        let target = config::Internal::target_kms();
        let mut client = KMSClient::new(target)?;
        let key_req = kms_proto::proto::GetKeyRequest::new(&file_info.key_id);
        let key_resp = client.get_key(key_req)?;
        let key_config = key_resp.get_key_config()?;
        let key_config = match key_config {
            kms_proto::KeyConfig::Aead(config) => kms_proto::proto::AeadConfig::from(config),
            kms_proto::KeyConfig::ProtectedFs(_config) => unimplemented!(), // ProtectedFS is not used by TDFS yet. Config of ProtectedFs will not be generated neither.
        };
        let access_path = &file_info.access_path;
        let mut f = fs::File::open(access_path)?;
        let capacity: usize = file_info.file_size.try_into().unwrap_or(1024 * 1024) + 1024;
        let mut ciphertxt: Vec<u8> = Vec::with_capacity(capacity);
        let mut buffer = vec![0; 1024 * 1024];
        while let Ok(bytes_len) = f.read(&mut buffer) {
            if bytes_len > 0 {
                ciphertxt.extend_from_slice(&buffer[0..bytes_len]);
            } else {
                break;
            }
        }

        let plaintxt = file_util::decrypt_data(
            ciphertxt,
            &key_config.key,
            &key_config.nonce,
            &key_config.ad,
        )?;
        Ok(plaintxt)
    }

    pub fn check_access_permission(&mut self, file_id: &str, user_id: &str) -> Result<bool> {
        let req = DFSRequest::new_check_permission(file_id, user_id);
        let resp = self.channel.invoke(req)?;
        match resp {
            DFSResponse::CheckUserPermission(resp) => Ok(resp.accessible),
            _ => Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::RPCResponseError,
            )),
        }
    }
}
