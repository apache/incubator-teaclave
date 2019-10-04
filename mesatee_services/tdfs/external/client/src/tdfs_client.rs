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

use crate::file_util;
use mesatee_core::config::{OutboundDesc, TargetDesc};
use mesatee_core::rpc::channel::SgxTrustedChannel;
use mesatee_core::{self, Result};
use std::fs;
use tdfs_external_proto::{
    CreateFileResponse, DFSRequest, DFSResponse, GetFileResponse, ListFileResponse,
};

pub struct TDFSClient {
    user_id: String,
    user_token: String,
    channel: SgxTrustedChannel<DFSRequest, DFSResponse>,
}

impl TDFSClient {
    pub fn new(target: &TargetDesc, user_id: &str, user_token: &str) -> Result<Self> {
        let addr = target.addr;

        let channel = match &target.desc {
            OutboundDesc::Sgx(enclave_attr) => {
                SgxTrustedChannel::<DFSRequest, DFSResponse>::new(addr, enclave_attr.clone())?
            }
        };

        Ok(TDFSClient {
            channel,
            user_id: user_id.to_string(),
            user_token: user_token.to_string(),
        })
    }

    fn request_create_file(
        &mut self,
        file_name: &str,
        sha256: &str,
        file_size: u32,
    ) -> Result<CreateFileResponse> {
        let req = DFSRequest::new_create_file(
            file_name,
            sha256,
            file_size,
            &self.user_id,
            &self.user_token,
        );
        let resp = self.channel.invoke(req)?;
        match resp {
            DFSResponse::Create(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::RPCResponseError,
            )),
        }
    }

    fn request_get_file(&mut self, file_id: &str) -> Result<GetFileResponse> {
        let req = DFSRequest::new_get_file(file_id, &self.user_id, &self.user_token);
        let resp = self.channel.invoke(req)?;
        match resp {
            DFSResponse::Get(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::RPCResponseError,
            )),
        }
    }

    pub fn save_file(&mut self, file_path: &str, file_name: &str) -> Result<String> {
        let data = fs::read(&file_path)
            .map_err(|_| mesatee_core::Error::from(mesatee_core::ErrorKind::IoError))?;
        let sha256 = file_util::cal_hash(&data)?;
        let file_size = data.len() as u32;
        let resp = self.request_create_file(file_name, &sha256, file_size)?;
        let file_id = resp.file_id;
        let access_path = resp.access_path;
        let key_config = resp.key_config;
        let encrypted_data =
            file_util::encrypt_data(data, &key_config.key, &key_config.nonce, &key_config.ad)?;
        fs::write(access_path, &encrypted_data)
            .map_err(|_| mesatee_core::Error::from(mesatee_core::ErrorKind::IoError))?;

        Ok(file_id)
    }

    pub fn read_file(&mut self, file_id: &str) -> Result<Vec<u8>> {
        let resp = self.request_get_file(file_id)?;
        let file_info = resp.file_info;
        let key_config = &file_info.key_config;
        let access_path = &file_info.access_path;
        let ciphertxt = fs::read(access_path)
            .map_err(|_| mesatee_core::Error::from(mesatee_core::ErrorKind::IoError))?;

        let plaintxt = file_util::decrypt_data(
            ciphertxt,
            &key_config.key,
            &key_config.nonce,
            &key_config.ad,
        )?;
        Ok(plaintxt)
    }

    pub fn request_list_file(&mut self) -> Result<ListFileResponse> {
        let req = DFSRequest::new_list_file(&self.user_id, &self.user_token);
        let resp = self.channel.invoke(req)?;
        match resp {
            DFSResponse::List(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::RPCResponseError,
            )),
        }
    }
}
