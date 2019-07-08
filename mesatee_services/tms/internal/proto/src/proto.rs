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

use serde_derive::*;
pub use tms_common_proto::{FunctionType, TaskFile, TaskInfo, TaskStatus};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TaskRequest {
    Get(GetTaskRequest),
    Update(UpdateTaskRequest),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TaskResponse {
    Get(GetTaskResponse),
    Update(UpdateTaskResponse),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetTaskRequest {
    pub task_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetTaskResponse {
    pub task_info: TaskInfo,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateTaskRequest {
    pub task_id: String,
    pub task_result_file_id: Option<String>,
    pub output_files: Vec<TaskFile>,
    pub status: Option<TaskStatus>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateTaskResponse {
    pub success: bool,
}

impl TaskRequest {
    pub fn new_get_task(task_id: &str) -> TaskRequest {
        let req = GetTaskRequest {
            task_id: task_id.to_owned(),
        };
        TaskRequest::Get(req)
    }

    pub fn new_update_task(
        task_id: &str,
        task_result_file_id: Option<&str>,
        output_files: &[&TaskFile],
        status: Option<&TaskStatus>,
    ) -> TaskRequest {
        let req = UpdateTaskRequest {
            task_id: task_id.to_owned(),
            task_result_file_id: task_result_file_id.map(|s| s.to_string()),
            output_files: output_files
                .iter()
                .map(|&task_file| task_file.clone())
                .collect(),
            status: status.copied(),
        };
        TaskRequest::Update(req)
    }
}

impl TaskResponse {
    pub fn new_update_task(success: bool) -> TaskResponse {
        let resp = UpdateTaskResponse { success };
        TaskResponse::Update(resp)
    }

    pub fn new_get_task(task_info: &TaskInfo) -> TaskResponse {
        let resp = GetTaskResponse {
            task_info: task_info.clone(),
        };
        TaskResponse::Get(resp)
    }
}
