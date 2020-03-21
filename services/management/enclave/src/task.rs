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
use anyhow::{anyhow, ensure, Result};
use std::collections::HashMap;
use std::collections::HashSet;
use std::prelude::v1::*;
use teaclave_types::Function;
use teaclave_types::{
    DataOwnerList, Storable, Task, TaskStatus, TeaclaveInputFile, TeaclaveOutputFile,
};
use uuid::Uuid;

pub(crate) fn create_task(
    function: Function,
    creator: String,
    arg_list: HashMap<String, String>,
    input_data_owner_list: HashMap<String, DataOwnerList>,
    output_data_owner_list: HashMap<String, DataOwnerList>,
) -> Result<Task> {
    let task_id = Uuid::new_v4();
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
    let function_arguments = arg_list.into();
    let task = Task {
        task_id,
        creator,
        function_id: function.external_id(),
        function_owner: function.owner,
        function_arguments,
        input_data_owner_list,
        output_data_owner_list,
        participants,
        approved_user_list: HashSet::new(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
        return_value: None,
        output_file_hash: HashMap::new(),
        status: TaskStatus::Created,
    };
    // check arguments
    let function_args: HashSet<String> = function.arg_list.into_iter().collect();
    let provide_args: HashSet<String> = task.function_arguments.inner().keys().cloned().collect();
    let diff: HashSet<_> = function_args.difference(&provide_args).collect();
    ensure!(diff.is_empty(), "bad arguments");

    // check input
    let input_args: HashSet<String> = function.input_list.into_iter().map(|f| f.name).collect();
    let provide_args: HashSet<String> = task.input_data_owner_list.keys().cloned().collect();
    let diff: HashSet<_> = input_args.difference(&provide_args).collect();
    ensure!(diff.is_empty(), "bad input");

    // check output
    let output_args: HashSet<String> = function.output_list.into_iter().map(|f| f.name).collect();
    let provide_args: HashSet<String> = task.output_data_owner_list.keys().cloned().collect();
    let diff: HashSet<_> = output_args.difference(&provide_args).collect();
    ensure!(diff.is_empty(), "bad output");

    Ok(task)
}

// access control:
// 1) input_file.owner contains user_id
// 2) input_data_owner_list contains the data name
// 3) user_id_list == input_file.owner
pub(crate) fn assign_input_to_task(
    task: &mut Task,
    data_name: &str,
    file: &TeaclaveInputFile,
    user_id: &str,
) -> Result<()> {
    if !file.owner.contains(&user_id.to_string()) {
        return Err(anyhow!("no permission"));
    }
    match task.input_data_owner_list.get(data_name) {
        Some(data_owner_list) => {
            let user_id_list = &data_owner_list.user_id_list;
            if user_id_list.len() != file.owner.len() {
                return Err(anyhow!("no permission"));
            }
            for owner in user_id_list.iter() {
                if !file.owner.contains(owner) {
                    return Err(anyhow!("no permission"));
                }
            }
        }
        None => return Err(anyhow!("no such input name")),
    };
    task.input_map
        .insert(data_name.to_owned(), file.external_id());
    Ok(())
}

// access control:
// 1) output_file is not used.
// 2) output_file.owner contains user_id
// 3) output_data_owner_list contains the data name
// 4) user_id_list == output_file.owner
pub(crate) fn assign_output_to_task(
    task: &mut Task,
    data_name: &str,
    file: &TeaclaveOutputFile,
    user_id: &str,
) -> Result<()> {
    if file.hash.is_some() {
        return Err(anyhow!("no permission"));
    }
    if !file.owner.contains(&user_id.to_string()) {
        return Err(anyhow!("no permission"));
    }
    match task.output_data_owner_list.get(data_name) {
        Some(data_owner_list) => {
            let user_id_list = &data_owner_list.user_id_list;
            if user_id_list.len() != file.owner.len() {
                return Err(anyhow!("no permission"));
            }
            for owner in user_id_list.iter() {
                if !file.owner.contains(owner) {
                    return Err(anyhow!("no permission"));
                }
            }
        }
        None => return Err(anyhow!("no such output name")),
    };
    task.output_map
        .insert(data_name.to_owned(), file.external_id());
    Ok(())
}

pub(crate) fn try_update_task_to_ready_status(task: &mut Task) {
    match task.status {
        TaskStatus::Created => {}
        _ => return,
    }

    // check input
    let input_args: HashSet<String> = task.input_data_owner_list.keys().cloned().collect();
    let assiged_inputs: HashSet<String> = task.input_map.keys().cloned().collect();
    let diff: HashSet<_> = input_args.difference(&assiged_inputs).collect();
    if !diff.is_empty() {
        return;
    }

    // check output
    let output_args: HashSet<String> = task.output_data_owner_list.keys().cloned().collect();
    let assiged_outputs: HashSet<String> = task.output_map.keys().cloned().collect();
    let diff: HashSet<_> = output_args.difference(&assiged_outputs).collect();
    if !diff.is_empty() {
        return;
    }
    task.status = TaskStatus::Ready;
}

pub(crate) fn try_update_task_to_approved_status(task: &mut Task) {
    match task.status {
        TaskStatus::Ready => {}
        _ => return,
    }
    let participants: HashSet<&String> = task.participants.iter().collect();
    let approved_users: HashSet<&String> = task.approved_user_list.iter().collect();

    let diff: HashSet<_> = participants.difference(&approved_users).collect();
    if !diff.is_empty() {
        return;
    }
    task.status = TaskStatus::Approved;
}
