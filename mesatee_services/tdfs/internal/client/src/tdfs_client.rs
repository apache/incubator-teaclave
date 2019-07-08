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
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::file_util;
use kms_client::KMSClient;
use mesatee_core::config::{self, OutboundDesc, TargetDesc};
use mesatee_core::rpc::channel::SgxTrustedChannel;
use mesatee_core::{self, Result};
use std::untrusted::fs;
use tdfs_internal_proto::{CreateFileResponse, DFSRequest, DFSResponse, FileInfo, GetFileResponse};

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

    fn request_create_file(
        &mut self,
        sha256: &str,
        file_size: u32,
        user_id: &str,
        task_id: &str,
        collaborator_list: &[&str],
        allow_policy: u32,
    ) -> Result<CreateFileResponse> {
        let req = DFSRequest::new_create_file(
            sha256,
            file_size,
            user_id,
            task_id,
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

    fn request_get_file(&mut self, file_id: &str) -> Result<GetFileResponse> {
        let req = DFSRequest::new_get_file(file_id);
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
            collaborator_list,
            allow_policy,
        )?;
        let file_id = resp.file_id;
        let access_path = resp.access_path;
        let key_config = resp.key_config;
        let encrypted_data =
            file_util::encrypt_data(data, &key_config.key, &key_config.nonce, &key_config.ad)?;
        fs::write(access_path, &encrypted_data)
            .map_err(|_| mesatee_core::Error::from(mesatee_core::ErrorKind::IoError))?;

        Ok(file_id)
    }

    fn check_permission(file_info: &FileInfo, user: &str) -> bool {
        let file_owner = &file_info.user_id;
        if (file_owner == user) || (file_info.allow_policy == 2) {
            return true;
        }
        if file_info.allow_policy == 1 {
            for collaborator_id in file_info.collaborator_list.iter() {
                if user == collaborator_id {
                    return true;
                }
            }
        }
        false
    }

    pub fn read_file(&mut self, file_id: &str, user_to_check: Option<&str>) -> Result<Vec<u8>> {
        let resp = self.request_get_file(file_id)?;
        let file_info = resp.file_info;

        let accessible: bool = match user_to_check {
            Some(ref user_id) => Self::check_permission(&file_info, user_id),
            None => true,
        };
        if !accessible {
            return Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::PermissionDenied,
            ));
        }

        let target = config::Internal::target_kms();
        let mut client = KMSClient::new(target)?;
        let key_resp = client.request_get_key(&file_info.key_id)?;
        let key_config = key_resp.config;
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

    pub fn check_access_permission(&mut self, file_id: &str, user_id: &str) -> Result<bool> {
        let resp = self.request_get_file(file_id)?;
        let file_info = resp.file_info;
        let accessible = Self::check_permission(&file_info, &user_id);
        Ok(accessible)
    }
}
