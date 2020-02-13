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
use crate::function::Function;
use anyhow::{anyhow, ensure, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::collections::HashSet;
use std::prelude::v1::*;
use teaclave_proto::teaclave_frontend_service::{DataOwnerList, TaskStatus};
use uuid::Uuid;

const TASK_PREFIX: &str = "task-";

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Task {
    pub(crate) task_id: String,
    pub(crate) creator: String,
    pub(crate) function_id: String,
    pub(crate) function_owner: String,
    pub(crate) arg_list: HashMap<String, String>,
    pub(crate) input_data_owner_list: HashMap<String, DataOwnerList>,
    pub(crate) output_data_owner_list: HashMap<String, DataOwnerList>,
    pub(crate) participants: HashSet<String>,
    pub(crate) approved_user_list: HashSet<String>,
    pub(crate) input_map: HashMap<String, String>,
    pub(crate) output_map: HashMap<String, String>,
    pub(crate) status: TaskStatus,
}

impl Task {
    pub(crate) fn new(
        function: Function,
        creator: String,
        arg_list: HashMap<String, String>,
        input_data_owner_list: HashMap<String, DataOwnerList>,
        output_data_owner_list: HashMap<String, DataOwnerList>,
    ) -> Result<Self> {
        let task_id = format!("{}{}", TASK_PREFIX, Uuid::new_v4().to_string());
        let mut participants = HashSet::new();
        if !function.is_public {
            participants.insert(function.owner.clone());
        }
        participants.insert(creator.clone());
        for (_, data_owner_list) in input_data_owner_list.iter() {
            for user_id in data_owner_list.user_id_list.iter() {
                participants.insert(user_id.clone());
            }
        }
        for (_, data_owner_list) in output_data_owner_list.iter() {
            for user_id in data_owner_list.user_id_list.iter() {
                participants.insert(user_id.clone());
            }
        }
        let task = Self {
            task_id,
            creator,
            function_id: function.function_id,
            function_owner: function.owner,
            arg_list,
            input_data_owner_list,
            output_data_owner_list,
            participants,
            approved_user_list: HashSet::new(),
            input_map: HashMap::new(),
            output_map: HashMap::new(),
            status: TaskStatus::Created,
        };
        // check arguments
        let function_args: HashSet<String> = function.arg_list.into_iter().collect();
        let provide_args: HashSet<String> = task.arg_list.keys().cloned().collect();
        let diff: HashSet<_> = function_args.difference(&provide_args).collect();
        ensure!(diff.is_empty(), "bad arguments");

        // check input
        let input_args: HashSet<String> = function.input_list.into_iter().map(|f| f.name).collect();
        let provide_args: HashSet<String> = task.input_data_owner_list.keys().cloned().collect();
        let diff: HashSet<_> = input_args.difference(&provide_args).collect();
        ensure!(diff.is_empty(), "bad input");

        // check output
        let output_args: HashSet<String> =
            function.output_list.into_iter().map(|f| f.name).collect();
        let provide_args: HashSet<String> = task.output_data_owner_list.keys().cloned().collect();
        let diff: HashSet<_> = output_args.difference(&provide_args).collect();
        ensure!(diff.is_empty(), "bad output");

        Ok(task)
    }

    pub(crate) fn from_slice(bytes: &[u8]) -> Result<Self> {
        let ret: Task =
            serde_json::from_slice(&bytes).map_err(|_| anyhow!("failed to Deserialize"))?;
        Ok(ret)
    }

    pub(crate) fn to_vec(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self).map_err(|_| anyhow!("failed to Serialize"))
    }

    pub(crate) fn get_key_vec(&self) -> Vec<u8> {
        self.task_id.as_bytes().to_vec()
    }

    pub(crate) fn is_task_id(id: &str) -> bool {
        id.starts_with(TASK_PREFIX)
    }
}
