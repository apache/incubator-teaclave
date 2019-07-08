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

use fns_proto::{InvokeTaskRequest, InvokeTaskResponse};
use mesatee_core::config::{OutboundDesc, TargetDesc};
use mesatee_core::rpc::channel::SgxTrustedChannel;
use mesatee_core::Result;

pub struct FNSClient {
    channel: SgxTrustedChannel<InvokeTaskRequest, InvokeTaskResponse>,
}

impl FNSClient {
    pub fn new(target: &TargetDesc) -> Result<Self> {
        let addr = target.addr;

        let channel = match &target.desc {
            OutboundDesc::Sgx(enclave_attr) => SgxTrustedChannel::<
                InvokeTaskRequest,
                InvokeTaskResponse,
            >::new(addr, enclave_attr.clone())?,
        };

        Ok(FNSClient { channel })
    }

    pub fn invoke_task(
        &mut self,
        task_id: &str,
        function_name: &str,
        task_token: &str,
        payload: Option<&str>,
    ) -> Result<InvokeTaskResponse> {
        let req = InvokeTaskRequest::new(task_id, function_name, task_token, payload);
        self.channel.invoke(req)
    }
}
