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

use crate::FunctionArguments;
use crate::Storable;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::prelude::v1::*;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DataOwnerList {
    pub user_id_list: HashSet<String>,
}

#[derive(Debug, Deserialize, Serialize, std::cmp::PartialEq)]
pub enum TaskStatus {
    Created,
    Ready,
    Approved,
    Running,
    Failed,
    Finished,
}

const TASK_PREFIX: &str = "task";

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub task_id: Uuid,
    pub creator: String,
    pub function_id: String,
    pub function_owner: String,
    pub function_arguments: FunctionArguments,
    pub input_data_owner_list: HashMap<String, DataOwnerList>,
    pub output_data_owner_list: HashMap<String, DataOwnerList>,
    pub participants: HashSet<String>,
    pub approved_user_list: HashSet<String>,
    pub input_map: HashMap<String, String>,
    pub output_map: HashMap<String, String>,
    pub return_value: Option<Vec<u8>>,
    pub output_file_hash: HashMap<String, String>,
    pub status: TaskStatus,
}

impl Storable for Task {
    fn key_prefix() -> &'static str {
        TASK_PREFIX
    }

    fn uuid(&self) -> Uuid {
        self.task_id
    }
}
