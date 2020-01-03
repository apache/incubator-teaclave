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

use serde_derive::*;
use std::net::IpAddr;

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum FunctionType {
    Single,
    Multiparty,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum TaskStatus {
    Created,
    Ready,
    Running,
    Finished,
    Failed,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct TaskFile {
    pub user_id: String,
    pub file_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct CollaboratorStatus {
    pub user_id: String,
    pub approved: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TaskInfo {
    pub user_id: String,
    pub collaborator_list: Vec<CollaboratorStatus>,
    pub approved_user_number: usize,
    pub function_name: String,
    pub function_type: FunctionType,
    pub status: TaskStatus,
    pub ip: IpAddr,
    pub port: u16,
    pub task_token: String,
    pub input_files: Vec<TaskFile>,
    pub output_files: Vec<TaskFile>,
    pub task_result_file_id: Option<String>,
}
