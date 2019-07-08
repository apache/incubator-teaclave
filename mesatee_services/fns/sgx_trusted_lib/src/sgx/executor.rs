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

use fns_proto::InvokeTaskRequest;
use mesatee_core::Result;
use std::collections::HashMap;
use tdfs_internal_client::TDFSClient;
use tms_internal_client::TMSClient;
use tms_internal_proto::{TaskFile, TaskInfo, TaskStatus};

use crate::trait_defs::{TaskExecutor, Worker, WorkerHelper, WorkerInput};
use crate::trusted_worker::TrustedWorker;

use mesatee_core::config;

struct TaskResult {
    output: Option<String>,
    output_files: Vec<TaskFile>,
    task_result_file_id: Option<String>,
}
pub struct Executor {
    task_info: TaskInfo,
    task_id: String,
    worker_input: WorkerInput,
    task_result: TaskResult,
    file_owner_map: HashMap<String, String>,
}

impl TaskExecutor for Executor {
    fn init(request: &InvokeTaskRequest) -> Result<Self> {
        let target = config::Internal::target_tms();
        let mut client = TMSClient::new(target)?;
        let resp = client.request_get_task(&request.task_id)?;
        let task_info = resp.task_info;

        // verify token
        if task_info.task_token != request.task_token {
            return Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::PermissionDenied,
            ));
        }

        // verify function name
        if task_info.function_name != request.function_name {
            return Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::PermissionDenied,
            ));
        }

        // verify status
        match &task_info.status {
            TaskStatus::Ready => {}
            _ => {
                return Err(mesatee_core::Error::from(
                    mesatee_core::ErrorKind::PermissionDenied,
                ))
            }
        }

        // Prepare task input
        let mut file_owner_map: HashMap<String, String> = HashMap::new();
        let mut input_files: Vec<String> = Vec::new();

        for task_file in task_info.input_files.iter() {
            input_files.push(task_file.file_id.to_string());
            file_owner_map.insert(task_file.file_id.to_string(), task_file.user_id.to_string());
        }

        let worker_input = WorkerInput {
            function_name: task_info.function_name.to_owned(),
            input_files: input_files,
            payload: request.payload.clone(),
        };

        // Todo: verify this is the expected worker

        // Init task result
        let task_result = TaskResult {
            output: None,
            output_files: Vec::new(),
            task_result_file_id: None,
        };

        Ok(Executor {
            task_info: task_info,
            task_id: request.task_id.to_owned(),
            worker_input: worker_input,
            task_result: task_result,
            file_owner_map: file_owner_map,
        })
    }
    fn execute(&mut self) -> Result<String> {
        // set the task status to `running`
        let target = config::Internal::target_tms();
        let mut client = TMSClient::new(target)?;
        let status = Some(&TaskStatus::Running);
        let _ = client.request_update_task(&self.task_id, None, &[], status)?;

        // Launch Worker
        let mut worker = TrustedWorker::new();
        worker.launch()?;

        // Do computation
        let result = worker.compute(self);
        if let Ok(ref value) = result {
            self.task_result.output = Some(value.to_string())
        };

        result
    }

    fn finalize(self) -> Result<()> {
        let target = config::Internal::target_tms();
        let mut client = TMSClient::new(target)?;

        match self.task_result.output {
            Some(_) => {
                let status = Some(&TaskStatus::Finished);
                let task_result_file_id = self.task_result.task_result_file_id.as_ref();
                let output_files: Vec<&TaskFile> = self.task_result.output_files.iter().collect();

                let _ = client.request_update_task(
                    &self.task_id,
                    task_result_file_id.map(|s| s.as_str()),
                    &output_files,
                    status,
                )?;
                Ok(())
            }
            None => {
                let status = Some(&TaskStatus::Failed);
                let _ = client.request_update_task(&self.task_id, None, &[], status)?;
                Ok(())
            }
        }
    }
}

impl Executor {
    pub fn save_file(
        &mut self,
        data: &[u8],
        user_id: &str,
        is_for_all_participants: bool,
    ) -> Result<String> {
        let allow_policy: u32;
        let collaborator_list: Vec<&str>;
        if is_for_all_participants {
            allow_policy = 1;
            collaborator_list = self
                .task_info
                .collaborator_list
                .iter()
                .map(|collorator_status| collorator_status.user_id.as_str())
                .collect();
        } else {
            allow_policy = 0;
            collaborator_list = Vec::new();
        };

        let target = config::Internal::target_tdfs();
        let mut client = TDFSClient::new(target)?;

        let file_id = client.save_file(
            data,
            user_id,
            &self.task_id,
            &collaborator_list,
            allow_policy,
        )?;

        if is_for_all_participants {
            self.task_result.task_result_file_id = Some(file_id.clone());
        } else {
            self.task_result.output_files.push(TaskFile {
                user_id: user_id.to_string(),
                file_id: file_id.to_string(),
            });
        }
        Ok(file_id)
    }
}
impl WorkerHelper for Executor {
    fn read_file(&mut self, file_id: &str) -> Result<Vec<u8>> {
        // TMS already checked
        let check_user_id = if self.file_owner_map.contains_key(file_id) {
            None
        } else {
            Some(self.task_info.user_id.as_str())
        };

        // Todo: Enforce other policies

        // read and check permission
        let target = config::Internal::target_tdfs();
        let mut client = TDFSClient::new(target)?;
        client.read_file(file_id, check_user_id)
    }

    fn get_input(&mut self) -> WorkerInput {
        self.worker_input.clone()
    }

    fn save_file_for_task_creator(&mut self, data: &[u8]) -> Result<String> {
        self.save_file(data, &self.task_info.user_id.to_string(), false)
    }
    fn save_file_for_all_participants(&mut self, data: &[u8]) -> Result<String> {
        self.save_file(data, &self.task_info.user_id.to_string(), true)
    }
    fn save_file_for_file_owner(&mut self, data: &[u8], file_id: &str) -> Result<String> {
        let file_owner: String = match self.file_owner_map.get(file_id) {
            Some(user_id) => user_id.to_string(),
            None => {
                return Err(mesatee_core::Error::from(
                    mesatee_core::ErrorKind::BadImplementation,
                ))
            }
        };

        self.save_file(data, &file_owner, false)
    }
}
