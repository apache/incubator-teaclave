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

use mesatee_core::config::{OutboundDesc, TargetDesc};
use mesatee_core::rpc::channel::SgxTrustedChannel;
use mesatee_core::{self, Result};
use tms_external_proto::{
    CreateTaskResponse, GetTaskResponse, ListTaskResponse, TaskRequest, TaskResponse,
    UpdateTaskResponse,
};

pub struct TMSClient {
    user_id: String,
    user_token: String,
    channel: SgxTrustedChannel<TaskRequest, TaskResponse>,
}

impl TMSClient {
    pub fn new(target: &TargetDesc, user_id: &str, user_token: &str) -> Result<Self> {
        let addr = target.addr;

        let channel = match &target.desc {
            OutboundDesc::Sgx(enclave_attr) => {
                SgxTrustedChannel::<TaskRequest, TaskResponse>::new(addr, enclave_attr.clone())?
            }
        };

        Ok(TMSClient {
            channel,
            user_id: user_id.to_string(),
            user_token: user_token.to_string(),
        })
    }

    pub fn request_create_task(
        &mut self,
        function_name: &str,
        collaborator_list: &[&str],
        files: &[&str],
    ) -> Result<CreateTaskResponse> {
        let req = TaskRequest::new_create_task(
            function_name,
            collaborator_list,
            files,
            &self.user_id,
            &self.user_token,
        );
        let resp = self.channel.invoke(req)?;
        match resp {
            TaskResponse::Create(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::RPCResponseError,
            )),
        }
    }

    pub fn request_get_task(&mut self, task_id: &str) -> Result<GetTaskResponse> {
        let req = TaskRequest::new_get_task(task_id, &self.user_id, &self.user_token);
        let resp = self.channel.invoke(req)?;
        match resp {
            TaskResponse::Get(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::RPCResponseError,
            )),
        }
    }

    pub fn request_update_task(
        &mut self,
        task_id: &str,
        files: &[&str],
    ) -> Result<UpdateTaskResponse> {
        let req = TaskRequest::new_update_task(task_id, files, &self.user_id, &self.user_token);
        let resp = self.channel.invoke(req)?;
        match resp {
            TaskResponse::Update(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::RPCResponseError,
            )),
        }
    }

    pub fn request_list_task(&mut self) -> Result<ListTaskResponse> {
        let req = TaskRequest::new_list_task(&self.user_id, &self.user_token);
        let resp = self.channel.invoke(req)?;
        match resp {
            TaskResponse::List(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::RPCResponseError,
            )),
        }
    }
}
