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

use uuid::Uuid;

use mesatee_core::config;
use mesatee_core::rpc::EnclaveService;
use mesatee_core::{Error, ErrorKind, Result};
use std::marker::PhantomData;
use tdfs_internal_client::TDFSClient;

use crate::data_store::{
    check_get_permission, gen_token, verify_user, CollaboratorStatus, FunctionType, TaskFile,
    TaskInfo, TaskStatus, TASK_STORE, UPDATELOCK,
};
use tms_external_proto::{
    CreateTaskRequest, GetTaskRequest, TaskRequest, TaskResponse, UpdateTaskRequest,
};

pub trait HandleRequest {
    fn handle_request(&self) -> Result<TaskResponse>;
}

impl HandleRequest for GetTaskRequest {
    fn handle_request(&self) -> Result<TaskResponse> {
        if !verify_user(&self.user_id, &self.user_token) {
            return Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::PermissionDenied,
            ));
        }

        let saved_info = TASK_STORE
            .get(&self.task_id)?
            .ok_or_else(|| Error::from(ErrorKind::MissingValue))?;

        // Check user permission
        if !check_get_permission(&saved_info, &self.user_id) {
            return Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::PermissionDenied,
            ));
        }

        // retrive current user's output files
        let mut output_files: Vec<String> = Vec::new();
        for task_file in saved_info.output_files.iter() {
            if task_file.user_id == self.user_id {
                output_files.push(task_file.file_id.to_owned());
            }
        }

        let return_info = tms_external_proto::TaskInfo {
            user_id: saved_info.user_id,
            function_name: saved_info.function_name,
            function_type: saved_info.function_type,
            status: saved_info.status,
            ip: saved_info.ip,
            port: saved_info.port,
            task_token: saved_info.task_token,
            collaborator_list: saved_info.collaborator_list,
            task_result_file_id: saved_info.task_result_file_id,
            user_private_result_file_id: output_files,
        };

        let resp = TaskResponse::new_get_task(&return_info);
        Ok(resp)
    }
}

impl HandleRequest for CreateTaskRequest {
    fn handle_request(&self) -> Result<TaskResponse> {
        if !verify_user(&self.user_id, &self.user_token) {
            return Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::PermissionDenied,
            ));
        }

        let func_type = match self.function_name.as_str() {
            "psi" | "concat" | "swap_file" | "private_join_and_compute" => FunctionType::Multiparty,
            _ => FunctionType::Single,
        };

        // check collaborator_list and files if func_type is Multiparty
        if let FunctionType::Multiparty = func_type {
            if self.collaborator_list.is_empty() {
                return Err(mesatee_core::Error::from(
                    mesatee_core::ErrorKind::MissingValue,
                ));
            }
            if self.files.is_empty() {
                return Err(mesatee_core::Error::from(
                    mesatee_core::ErrorKind::MissingValue,
                ));
            }
        }

        // check file permission
        for file_id in self.files.iter() {
            let target = config::Internal::target_tdfs();
            let mut client = TDFSClient::new(target)?;
            let accessible = client.check_access_permission(file_id, &self.user_id)?;
            if !accessible {
                return Err(mesatee_core::Error::from(
                    mesatee_core::ErrorKind::PermissionDenied,
                ));
            }
        }

        let collaborator_list: Vec<CollaboratorStatus> = self
            .collaborator_list
            .iter()
            .map(|user_id| CollaboratorStatus {
                user_id: user_id.to_string(),
                approved: false,
            })
            .collect();

        let token = gen_token()?;
        let input_files: Vec<TaskFile> = self
            .files
            .iter()
            .map(|file_id| TaskFile {
                user_id: self.user_id.to_string(),
                file_id: file_id.to_string(),
            })
            .collect();
        let fns_config = config::External::target_fns();
        let mut task_info = TaskInfo {
            user_id: self.user_id.to_string(),
            collaborator_list,
            approved_user_number: 0,
            function_name: self.function_name.to_string(),
            function_type: func_type,
            status: TaskStatus::Created,
            ip: fns_config.addr.ip(),
            port: fns_config.addr.port(),
            task_token: token,
            input_files,
            output_files: Vec::new(),
            task_result_file_id: None,
        };

        match func_type {
            FunctionType::Single => {
                task_info.status = TaskStatus::Ready;
            }
            _ => {
                task_info.status = TaskStatus::Created;
            }
        }

        let task_id = Uuid::new_v4().to_string();
        if TASK_STORE.get(&task_id)?.is_some() {
            return Err(Error::from(ErrorKind::UUIDError));
        }
        TASK_STORE.set(&task_id, &task_info)?;

        let resp = TaskResponse::new_create_task(
            &task_id,
            &task_info.task_token,
            task_info.ip,
            task_info.port,
        );
        Ok(resp)
    }
}

impl HandleRequest for UpdateTaskRequest {
    fn handle_request(&self) -> Result<TaskResponse> {
        if !verify_user(&self.user_id, &self.user_token) {
            return Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::PermissionDenied,
            ));
        }

        if self.files.is_empty() {
            return Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::MissingValue,
            ));
        }

        let task_info = TASK_STORE.get(&self.task_id)?;
        let task_info = match task_info {
            Some(value) => value,
            None => {
                return Err(mesatee_core::Error::from(
                    mesatee_core::ErrorKind::PermissionDenied,
                ))
            }
        };

        match task_info.function_type {
            FunctionType::Multiparty => {}
            _ => {
                return Err(mesatee_core::Error::from(
                    mesatee_core::ErrorKind::PermissionDenied,
                ))
            }
        }

        match task_info.status {
            TaskStatus::Created => {}
            _ => {
                return Err(mesatee_core::Error::from(
                    mesatee_core::ErrorKind::PermissionDenied,
                ))
            }
        }

        let _lock = UPDATELOCK.lock()?;
        let task_info = TASK_STORE.get(&self.task_id)?;
        let mut task_info = task_info.ok_or_else(|| mesatee_core::ErrorKind::PermissionDenied)?;
        // updated approved list
        let mut is_collaborator: bool = false;
        for collaborator in task_info.collaborator_list.iter_mut() {
            if collaborator.user_id == self.user_id && !collaborator.approved {
                collaborator.approved = true;
                is_collaborator = true;
                task_info.approved_user_number += 1;
                break;
            }
        }
        if !is_collaborator {
            return Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::PermissionDenied,
            ));
        }

        // update task status
        if task_info.approved_user_number == task_info.collaborator_list.len() {
            task_info.status = TaskStatus::Ready;
        }

        // Verify file permissions and update input files
        let target = config::Internal::target_tdfs();
        let mut client = TDFSClient::new(target)?;
        for file_id in self.files.iter() {
            let accessible = client.check_access_permission(&file_id, &self.user_id)?;
            if !accessible {
                return Err(mesatee_core::Error::from(
                    mesatee_core::ErrorKind::PermissionDenied,
                ));
            }
            task_info.input_files.push(TaskFile {
                user_id: self.user_id.to_string(),
                file_id: file_id.to_string(),
            });
        }

        TASK_STORE.set(&self.task_id, &task_info)?;

        let resp = TaskResponse::new_update_task(
            true,
            task_info.status,
            task_info.ip,
            task_info.port,
            &task_info.task_token,
        );

        Ok(resp)
    }
}

pub struct TMSExternalEnclave<S, T> {
    state: i32,
    x: PhantomData<S>,
    y: PhantomData<T>,
}

impl<S, T> Default for TMSExternalEnclave<S, T> {
    fn default() -> Self {
        TMSExternalEnclave {
            state: 0,
            x: PhantomData::<S>,
            y: PhantomData::<T>,
        }
    }
}

impl EnclaveService<TaskRequest, TaskResponse> for TMSExternalEnclave<TaskRequest, TaskResponse> {
    fn handle_invoke(&mut self, input: TaskRequest) -> Result<TaskResponse> {
        trace!("handle_invoke invoked!");
        trace!("incoming payload = {:?}", input);
        self.state += 1;
        let response = match input {
            TaskRequest::Create(req) => req.handle_request()?,
            TaskRequest::Get(req) => req.handle_request()?,
            TaskRequest::Update(req) => req.handle_request()?,
        };
        trace!("{}th round complete!", self.state);
        Ok(response)
    }
}
