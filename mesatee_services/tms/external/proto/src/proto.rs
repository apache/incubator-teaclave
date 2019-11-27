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

use crate::{CollaboratorStatus, FunctionType, TaskStatus};
use serde_derive::*;
use std::net::IpAddr;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TaskRequest {
    Get(GetTaskRequest),
    Create(CreateTaskRequest),
    Update(UpdateTaskRequest),
    List(ListTaskRequest),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TaskResponse {
    Get(GetTaskResponse),
    Create(CreateTaskResponse),
    Update(UpdateTaskResponse),
    List(ListTaskResponse),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetTaskRequest {
    pub task_id: String,
    pub user_id: String,
    pub user_token: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ListTaskRequest {
    pub user_id: String,
    pub user_token: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TaskInfo {
    pub user_id: String,
    pub function_name: String,
    pub function_type: FunctionType,
    pub status: TaskStatus,
    pub ip: IpAddr,
    pub port: u16,
    pub task_token: String,
    pub collaborator_list: Vec<CollaboratorStatus>,
    pub task_result_file_id: Option<String>,
    pub user_private_result_file_id: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetTaskResponse {
    pub task_info: TaskInfo,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CreateTaskRequest {
    pub function_name: String,
    pub collaborator_list: Vec<String>,
    pub files: Vec<String>,
    pub user_id: String,
    pub user_token: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CreateTaskResponse {
    pub task_id: String,
    pub task_token: String,
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateTaskRequest {
    pub task_id: String,
    pub files: Vec<String>,
    pub user_id: String,
    pub user_token: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateTaskResponse {
    pub success: bool,
    pub status: TaskStatus,
    pub ip: IpAddr,
    pub port: u16,
    pub task_token: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ListTaskResponse {
    pub list: Vec<String>,
}

impl TaskRequest {
    pub fn new_get_task(task_id: &str, user_id: &str, user_token: &str) -> TaskRequest {
        TaskRequest::Get(GetTaskRequest {
            task_id: task_id.to_owned(),
            user_id: user_id.to_owned(),
            user_token: user_token.to_owned(),
        })
    }

    pub fn new_create_task(
        function_name: &str,
        collaborator_list: &[&str],
        files: &[&str],
        user_id: &str,
        user_token: &str,
    ) -> TaskRequest {
        TaskRequest::Create(CreateTaskRequest {
            function_name: function_name.to_owned(),
            collaborator_list: collaborator_list.iter().map(|s| s.to_string()).collect(),
            files: files.iter().map(|s| s.to_string()).collect(),
            user_id: user_id.to_owned(),
            user_token: user_token.to_owned(),
        })
    }

    pub fn new_update_task(
        task_id: &str,
        files: &[&str],
        user_id: &str,
        user_token: &str,
    ) -> TaskRequest {
        TaskRequest::Update(UpdateTaskRequest {
            task_id: task_id.to_owned(),
            files: files.iter().map(|s| s.to_string()).collect(),
            user_id: user_id.to_owned(),
            user_token: user_token.to_owned(),
        })
    }

    pub fn new_list_task(user_id: &str, user_token: &str) -> TaskRequest {
        TaskRequest::List(ListTaskRequest {
            user_id: user_id.to_owned(),
            user_token: user_token.to_owned(),
        })
    }
}

impl TaskResponse {
    pub fn new_get_task(task_info: &TaskInfo) -> TaskResponse {
        TaskResponse::Get(GetTaskResponse {
            task_info: task_info.clone(),
        })
    }

    pub fn new_create_task(task_id: &str, task_token: &str, ip: IpAddr, port: u16) -> TaskResponse {
        TaskResponse::Create(CreateTaskResponse {
            task_id: task_id.to_owned(),
            task_token: task_token.to_owned(),
            ip,
            port,
        })
    }

    pub fn new_update_task(
        success: bool,
        status: TaskStatus,
        ip: IpAddr,
        port: u16,
        task_token: &str,
    ) -> TaskResponse {
        TaskResponse::Update(UpdateTaskResponse {
            success,
            status,
            ip,
            port,
            task_token: task_token.to_owned(),
        })
    }

    pub fn new_list_task(list: &[&str]) -> TaskResponse {
        TaskResponse::List(ListTaskResponse {
            list: list.iter().map(|s| s.to_string()).collect(),
        })
    }
}
