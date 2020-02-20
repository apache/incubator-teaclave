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
use crate::fusion_data::FusionData;
use anyhow::{anyhow, ensure, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::collections::HashSet;
use std::prelude::v1::*;
use teaclave_proto::teaclave_frontend_service::{DataOwnerList, TaskStatus};
use teaclave_types::TeaclaveFileCryptoInfo;
use teaclave_types::{Storable, TeaclaveInputFile, TeaclaveOutputFile};
use url::Url;
use uuid::Uuid;

const TASK_PREFIX: &str = "task-";
const STAGED_TASK_PREFIX: &str = "staged-"; // staged-task-uuid
const QUEUE_KEY: &str = "staged-task";
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

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct InputData {
    pub(crate) url: Url,
    pub(crate) hash: String,
    pub(crate) crypto_info: TeaclaveFileCryptoInfo,
}
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct OutputData {
    pub(crate) url: Url,
    pub(crate) crypto_info: TeaclaveFileCryptoInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct StagedTask {
    pub(crate) staged_task_id: String,
    pub(crate) task_id: String,
    pub(crate) function_id: String,
    pub(crate) function_payload: Vec<u8>,
    pub(crate) arg_list: HashMap<String, String>,
    pub(crate) input_map: HashMap<String, InputData>,
    pub(crate) output_map: HashMap<String, OutputData>,
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

    // access control:
    // 1) user_id == input_file.owner
    // 2) input_data_owner_list contains the data name
    // 3) DataOwnerList has only one user
    // 4) user_id_list == user_id
    pub(crate) fn assign_input_file(
        &mut self,
        data_name: &str,
        file: &TeaclaveInputFile,
        user_id: &str,
    ) -> Result<()> {
        if file.owner != user_id {
            return Err(anyhow!("no permission"));
        }
        match self.input_data_owner_list.get(data_name) {
            Some(data_owner_list) => {
                let user_id_list = &data_owner_list.user_id_list;
                if user_id_list.len() != 1 {
                    return Err(anyhow!("no permission"));
                }
                if !user_id_list.contains(user_id) {
                    return Err(anyhow!("no permission"));
                }
            }
            None => return Err(anyhow!("no such this input name")),
        };
        self.input_map
            .insert(data_name.to_owned(), file.uuid.to_string());
        Ok(())
    }

    // access control:
    // 1) output_file is not used.
    // 2) user_id == output_file.owner
    // 3) output_data_owner_list contains the data name
    // 4) DataOwnerList has only one user
    // 5) user_id_list == user_id
    pub(crate) fn assign_output_file(
        &mut self,
        data_name: &str,
        file: &TeaclaveOutputFile,
        user_id: &str,
    ) -> Result<()> {
        if file.hash.is_some() {
            return Err(anyhow!("no permission"));
        }
        if file.owner != user_id {
            return Err(anyhow!("no permission"));
        }
        match self.output_data_owner_list.get(data_name) {
            Some(data_owner_list) => {
                let user_id_list = &data_owner_list.user_id_list;
                if user_id_list.len() != 1 {
                    return Err(anyhow!("no permission"));
                }
                if !user_id_list.contains(user_id) {
                    return Err(anyhow!("no permission"));
                }
            }
            None => return Err(anyhow!("no such this input name")),
        };
        self.output_map
            .insert(data_name.to_owned(), file.external_id());
        Ok(())
    }

    // access control:
    // 1) fusion_data: fusion_data.owner_id_list.contains(user_id)
    // 2) DataOwnerList == fusion_data.owner_id_list
    // 3) input_data_owner_list contains the data name
    pub(crate) fn assign_fusion_data(
        &mut self,
        data_name: &str,
        fusion_data: &FusionData,
        user_id: &str,
    ) -> Result<()> {
        if !fusion_data
            .data_owner_id_list
            .contains(&user_id.to_string())
        {
            return Err(anyhow!("no permission"));
        }
        match self.input_data_owner_list.get(data_name) {
            Some(data_owner_list) => {
                let user_id_list = &data_owner_list.user_id_list;
                if user_id_list.len() != fusion_data.data_owner_id_list.len() {
                    return Err(anyhow!("no permission"));
                }
                for owner in user_id_list.iter() {
                    if !fusion_data.data_owner_id_list.contains(owner) {
                        return Err(anyhow!("no permission"));
                    }
                }
            }
            None => return Err(anyhow!("no such this input name")),
        }
        self.input_map
            .insert(data_name.to_string(), fusion_data.data_id.to_owned());
        Ok(())
    }

    pub(crate) fn is_task_id(id: &str) -> bool {
        id.starts_with(TASK_PREFIX)
    }

    pub(crate) fn try_update_to_ready_status(&mut self) {
        match self.status {
            TaskStatus::Created => {}
            _ => return,
        }

        // check input
        let input_args: HashSet<String> = self.input_data_owner_list.keys().cloned().collect();
        let assiged_inputs: HashSet<String> = self.input_map.keys().cloned().collect();
        let diff: HashSet<_> = input_args.difference(&assiged_inputs).collect();
        if !diff.is_empty() {
            return;
        }

        // check output
        let output_args: HashSet<String> = self.output_data_owner_list.keys().cloned().collect();
        let assiged_outputs: HashSet<String> = self.output_map.keys().cloned().collect();
        let diff: HashSet<_> = output_args.difference(&assiged_outputs).collect();
        if !diff.is_empty() {
            return;
        }
        self.status = TaskStatus::Ready;
    }

    pub(crate) fn try_update_to_approved_status(&mut self) {
        match self.status {
            TaskStatus::Ready => {}
            _ => return,
        }
        let participants: HashSet<&String> = self.participants.iter().collect();
        let approved_users: HashSet<&String> = self.approved_user_list.iter().collect();

        let diff: HashSet<_> = participants.difference(&approved_users).collect();
        if !diff.is_empty() {
            return;
        }
        self.status = TaskStatus::Approved;
    }
}

impl StagedTask {
    pub(crate) fn new(
        task_id: &str,
        function: Function,
        arg_list: HashMap<String, String>,
        input_map: HashMap<String, InputData>,
        output_map: HashMap<String, OutputData>,
    ) -> Self {
        Self {
            staged_task_id: format!("{}{}", STAGED_TASK_PREFIX, task_id),
            task_id: task_id.to_owned(),
            function_id: function.function_id,
            function_payload: function.payload,
            arg_list,
            input_map,
            output_map,
        }
    }

    pub(crate) fn get_queue_key() -> Vec<u8> {
        QUEUE_KEY.as_bytes().to_vec()
    }

    #[cfg(any(test_mod, feature = "enclave_unit_test"))]
    pub(crate) fn from_slice(bytes: &[u8]) -> Result<Self> {
        let ret: StagedTask =
            serde_json::from_slice(&bytes).map_err(|_| anyhow!("failed to Deserialize"))?;
        Ok(ret)
    }

    pub(crate) fn to_vec(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self).map_err(|_| anyhow!("failed to Serialize"))
    }
}

impl InputData {
    pub(crate) fn from_input_file(input: TeaclaveInputFile) -> Result<InputData> {
        Ok(InputData {
            url: input.url,
            hash: input.hash,
            crypto_info: input.crypto_info,
        })
    }

    pub(crate) fn from_fusion_data(input: FusionData) -> Result<InputData> {
        let hash = input.hash.ok_or_else(|| anyhow!("invalid fusion data"))?;
        Ok(InputData {
            url: input.url,
            hash,
            crypto_info: input.crypto_info,
        })
    }
}

impl OutputData {
    pub(crate) fn from_output_file(output: TeaclaveOutputFile) -> Result<OutputData> {
        if output.hash.is_some() {
            return Err(anyhow!("invalid output file"));
        }
        Ok(OutputData {
            url: output.url,
            crypto_info: output.crypto_info,
        })
    }
    pub(crate) fn from_fusion_data(output: FusionData) -> Result<OutputData> {
        if output.hash.is_some() {
            return Err(anyhow!("invalid fusion data"));
        }
        Ok(OutputData {
            url: output.url,
            crypto_info: output.crypto_info,
        })
    }
}
