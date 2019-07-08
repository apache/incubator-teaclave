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

use crate::data_store::TASK_STORE;
use mesatee_core::rpc::EnclaveService;
use mesatee_core::{Error, ErrorKind, Result};
use std::marker::PhantomData;
use tms_internal_proto::{GetTaskRequest, TaskRequest, TaskResponse, UpdateTaskRequest};

pub trait HandleRequest {
    fn handle_request(&self) -> Result<TaskResponse>;
}

impl HandleRequest for UpdateTaskRequest {
    fn handle_request(&self) -> Result<TaskResponse> {
        let old_info = TASK_STORE.get(&self.task_id)?;
        // ToDo: Race Condition can be solved in FNS
        let mut old_info = match old_info {
            Some(value) => value,
            None => return Ok(TaskResponse::new_update_task(false)),
        };

        if self.task_result_file_id.is_some() {
            old_info.task_result_file_id = self.task_result_file_id.clone();
        }

        old_info.output_files.extend_from_slice(&self.output_files);

        if let Some(ref status) = self.status {
            old_info.status = *status;
        }

        let _ = TASK_STORE.set(&self.task_id, &old_info)?;

        let resp = TaskResponse::new_update_task(true);
        Ok(resp)
    }
}

impl HandleRequest for GetTaskRequest {
    fn handle_request(&self) -> Result<TaskResponse> {
        let task_info = TASK_STORE
            .get(&self.task_id)?
            .ok_or_else(|| Error::from(ErrorKind::MissingValue))?;
        let resp = TaskResponse::new_get_task(&task_info);
        Ok(resp)
    }
}

pub struct TMSInternalEnclave<S, T> {
    state: i32,
    x: PhantomData<S>,
    y: PhantomData<T>,
}

impl<S, T> Default for TMSInternalEnclave<S, T> {
    fn default() -> Self {
        TMSInternalEnclave {
            state: 0,
            x: PhantomData::<S>,
            y: PhantomData::<T>,
        }
    }
}

impl EnclaveService<TaskRequest, TaskResponse> for TMSInternalEnclave<TaskRequest, TaskResponse> {
    fn handle_invoke(&mut self, input: TaskRequest) -> Result<TaskResponse> {
        trace!("handle_invoke invoked!");
        trace!("incoming payload = {:?}", input);
        self.state += 1;
        let response = match input {
            TaskRequest::Update(req) => req.handle_request()?,
            TaskRequest::Get(req) => req.handle_request()?,
        };
        trace!("{}th round complete!", self.state);
        Ok(response)
    }
}
