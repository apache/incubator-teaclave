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

// Insert std prelude in the top for the sgx feature
use fns_proto::InvokeTaskRequest;
use lazy_static::lazy_static;
use mesatee_core::{config, Error, ErrorKind, Result};
use std::collections::HashMap;
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;
use std::sync::Arc;
#[cfg(feature = "mesalock_sgx")]
use std::sync::SgxRwLock as RwLock;
use tdfs_internal_client::TDFSClient;
use tms_internal_client::TMSClient;
use tms_internal_proto::{TaskFile, TaskInfo, TaskStatus};
use uuid::Uuid;

use crate::worker::WorkerContext;

struct TaskResult {
    output: Option<String>,
    output_files: Vec<TaskFile>,
    task_result_file_id: Option<String>,
}

lazy_static! {
    static ref RUNNING_TASK_QUEUE: RwLock<HashMap<String, RunningTask>> =
        { RwLock::new(HashMap::new()) };
}

#[derive(Clone)]
pub struct RunningTask {
    task_info: TaskInfo,
    task_id: String,
    context_token: String,
    task_result: Arc<RwLock<TaskResult>>,
    file_owner_map: HashMap<String, String>,
}

impl RunningTask {
    pub fn init(request: &InvokeTaskRequest) -> Result<Self> {
        let target = config::Internal::target_tms();
        let mut client = TMSClient::new(target)?;
        let resp = client.request_get_task(&request.task_id)?;
        let task_info = resp.task_info;
        let task_id = request.task_id.to_owned();

        // verify token
        if task_info.task_token != request.task_token {
            return Err(Error::from(ErrorKind::PermissionDenied));
        }

        // verify function name
        if task_info.function_name != request.function_name {
            return Err(Error::from(ErrorKind::PermissionDenied));
        }

        // verify status
        match &task_info.status {
            TaskStatus::Ready => {}
            _ => return Err(Error::from(ErrorKind::PermissionDenied)),
        }

        // Prepare task input
        let mut file_owner_map: HashMap<String, String> = HashMap::new();
        for task_file in task_info.input_files.iter() {
            file_owner_map.insert(task_file.file_id.to_string(), task_file.user_id.to_string());
        }

        // Prepare ContextInfo
        let context_token = Uuid::new_v4().to_string();

        // Init task result
        let task_result = TaskResult {
            output: None,
            output_files: Vec::new(),
            task_result_file_id: None,
        };

        let running_task = RunningTask {
            task_info,
            task_id: task_id.to_owned(),
            context_token,
            task_result: Arc::new(RwLock::new(task_result)),
            file_owner_map,
        };

        // Todo: verify this is the expected worker

        // Add the running task to the queue
        {
            let mut queue = RUNNING_TASK_QUEUE.write()?;
            if queue.contains_key(&task_id) {
                return Err(Error::from(ErrorKind::PermissionDenied));
            }
            queue.insert(task_id.to_string(), running_task.clone());
        }
        // Set task status to "running" in TMS
        let status = Some(&TaskStatus::Running);
        let result = client.request_update_task(&task_id, None, &[], status);
        match result {
            Ok(_) => Ok(running_task),
            Err(err) => {
                let mut queue = RUNNING_TASK_QUEUE.write()?;
                queue.remove(&task_id);
                Err(err)
            }
        }
    }

    pub fn get_file_list(&self) -> Vec<String> {
        self.task_info
            .input_files
            .iter()
            .map(|file| file.file_id.to_owned())
            .collect()
    }

    fn remove_from_queue(task_id: &str) -> Result<()> {
        let mut queue = RUNNING_TASK_QUEUE.write()?;
        queue.remove(task_id);
        Ok(())
    }

    pub fn save_dynamic_output(&self, output: &str) -> Result<()> {
        let mut task_result = self.task_result.write()?;
        task_result.output = Some(output.to_string());
        Ok(())
    }

    pub fn finish(self) -> Result<()> {
        let _ = Self::remove_from_queue(&self.task_id);

        let task_result = self.task_result.read()?;
        let target = config::Internal::target_tms();
        let mut client = TMSClient::new(target)?;

        match task_result.output {
            Some(_) => {
                let status = Some(&TaskStatus::Finished);
                let task_result_file_id = task_result.task_result_file_id.as_ref();
                let output_files: Vec<&TaskFile> = task_result.output_files.iter().collect();

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

    pub fn get_worker_context(&self) -> WorkerContext {
        WorkerContext {
            context_id: self.task_id.to_owned(),
            context_token: self.context_token.to_owned(),
        }
    }
    pub fn retrieve_running_task(context_id: &str, context_token: &str) -> Result<RunningTask> {
        let queue = RUNNING_TASK_QUEUE.read()?;
        let task = queue
            .get(context_id)
            .ok_or_else(|| Error::from(ErrorKind::MissingValue))?;
        if task.context_token.as_str() != context_token {
            Err(Error::from(ErrorKind::PermissionDenied))
        } else {
            Ok(task.clone())
        }
    }

    fn save_file(
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

        let mut task_result = self.task_result.write()?;
        if is_for_all_participants {
            task_result.task_result_file_id = Some(file_id.clone());
        } else {
            task_result.output_files.push(TaskFile {
                user_id: user_id.to_string(),
                file_id: file_id.to_string(),
            });
        }
        Ok(file_id)
    }

    pub fn read_file(&mut self, file_id: &str) -> Result<Vec<u8>> {
        // read and check permission
        let target = config::Internal::target_tdfs();
        let mut client = TDFSClient::new(target)?;
        client.read_file(file_id, &self.task_id, &self.task_info.task_token)
    }

    pub fn save_file_for_task_creator(&mut self, data: &[u8]) -> Result<String> {
        self.save_file(data, &self.task_info.user_id.to_string(), false)
    }
    pub fn save_file_for_all_participants(&mut self, data: &[u8]) -> Result<String> {
        self.save_file(data, &self.task_info.user_id.to_string(), true)
    }
    pub fn save_file_for_file_owner(&mut self, data: &[u8], file_id: &str) -> Result<String> {
        let file_owner: String = match self.file_owner_map.get(file_id) {
            Some(user_id) => user_id.to_string(),
            None => return Err(Error::from(ErrorKind::BadImplementation)),
        };

        self.save_file(data, &file_owner, false)
    }
}
