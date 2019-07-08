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

use kms_proto::{CreateKeyResponse, GetKeyResponse, KMSRequest, KMSResponse};
use mesatee_core::config::{OutboundDesc, TargetDesc};
use mesatee_core::rpc::channel::SgxTrustedChannel;
use mesatee_core::{Error, ErrorKind, Result};

pub struct KMSClient {
    channel: SgxTrustedChannel<KMSRequest, KMSResponse>,
}

impl KMSClient {
    pub fn new(target: TargetDesc) -> Result<Self> {
        let addr = target.addr;
        let channel = match target.desc {
            OutboundDesc::Sgx(enclave_addr) => {
                SgxTrustedChannel::<KMSRequest, KMSResponse>::new(addr, enclave_addr)?
            }
        };
        Ok(KMSClient { channel })
    }

    pub fn request_create_key(&mut self) -> Result<CreateKeyResponse> {
        let req = KMSRequest::new_create_key();
        let resp = self.channel.invoke(req)?;
        match resp {
            KMSResponse::Create(resp) => Ok(resp),
            _ => Err(Error::from(ErrorKind::RPCResponseError)),
        }
    }

    pub fn request_get_key(&mut self, key_id: &str) -> Result<GetKeyResponse> {
        let req = KMSRequest::new_get_key(key_id);
        let resp = self.channel.invoke(req)?;
        match resp {
            KMSResponse::Get(resp) => Ok(resp),
            _ => Err(Error::from(ErrorKind::RPCResponseError)),
        }
    }
}
