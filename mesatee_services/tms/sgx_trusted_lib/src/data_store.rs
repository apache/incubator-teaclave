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

use mesatee_core::db::Memdb;
use mesatee_core::Result;
use std::collections::HashSet;
use std::fmt::Write;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::SgxMutex;

pub use tms_common_proto::CollaboratorStatus;
pub use tms_common_proto::FunctionType;
pub use tms_common_proto::TaskFile;
pub use tms_common_proto::TaskInfo;
pub use tms_common_proto::TaskStatus;

use lazy_static::lazy_static;

lazy_static! {
    // At this moment, this is just a workaround;
    // Later the data structure will be redesigned according to the persistent db, lock mechanism  and architecture.
    pub static ref USER_TASK_STORE: Memdb<String, HashSet<String>> = {
        Memdb::<String, HashSet<String>>::open().expect("cannot open db")
    };
    pub static ref TASK_STORE: Memdb<String, TaskInfo> = {
        Memdb::<String, TaskInfo>::open().expect("cannot open db")
    };

    pub static ref UPDATELOCK: SgxMutex<u32> = SgxMutex::new(0);
}

pub fn verify_user(_user_id: &str, user_token: &str) -> bool {
    if user_token == "error_token" {
        return false;
    }
    true
}

pub fn check_get_permission(task_info: &TaskInfo, user_id: &str) -> bool {
    if task_info.user_id == user_id {
        return true;
    }
    for collaborator in task_info.collaborator_list.iter() {
        if user_id == collaborator.user_id {
            return true;
        }
    }
    false
}

pub fn check_task_token(task_info: &TaskInfo, task_token: &str) -> bool {
    task_info.task_token == task_token
}

pub fn gen_token() -> Result<String> {
    use rand::prelude::RngCore;
    let mut token: [u8; 16] = [0; 16];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut token);
    let mut hex_token = String::new();
    for &byte in &token {
        write!(&mut hex_token, "{:02x}", byte)
            .map_err(|_| mesatee_core::Error::from(mesatee_core::ErrorKind::Unknown))?;
    }
    Ok(hex_token)
}

// Before calling this function, use lock to avoid data race;
fn add_task_to_user(task_id: &str, user_id: &str) -> Result<()> {
    let uid = user_id.to_owned();
    let id_set = USER_TASK_STORE.get(&uid)?;
    match id_set {
        Some(mut set) => {
            set.insert(task_id.to_owned());
            USER_TASK_STORE.set(&uid, &set)?;
        }
        None => {
            let mut set = HashSet::<String>::new();
            set.insert(task_id.to_owned());
            USER_TASK_STORE.set(&uid, &set)?;
        }
    }
    Ok(())
}
pub fn add_task(task_id: &str, task_info: &TaskInfo) -> Result<()> {
    let _ = TASK_STORE.set(&task_id.to_owned(), &task_info)?;
    let _lock = UPDATELOCK.lock()?;
    add_task_to_user(task_id, &task_info.user_id)?;
    for collaborator in task_info.collaborator_list.iter() {
        add_task_to_user(task_id, &collaborator.user_id)?;
    }
    Ok(())
}

// For API Test, called by enclave_init
pub fn add_test_infomation() {
    let fake_task = TaskInfo {
        user_id: "fake".to_string(),
        collaborator_list: Vec::new(),
        approved_user_number: 0,
        function_name: "echo".to_string(),
        function_type: FunctionType::Single,
        status: TaskStatus::Ready,
        ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        port: 0,
        task_token: "fake".to_string(),
        input_files: Vec::new(),
        output_files: Vec::new(),
        task_result_file_id: None,
    };
    let _ = add_task(&"fake".to_owned(), &fake_task);

    let collaborator_for_fake_task = CollaboratorStatus {
        user_id: "fake_file_owner".to_string(),
        approved: false,
    };

    let fake_multi_task = TaskInfo {
        user_id: "fake".to_string(),
        collaborator_list: vec![collaborator_for_fake_task],
        approved_user_number: 0,
        function_name: "fake".to_string(),
        function_type: FunctionType::Multiparty,
        status: TaskStatus::Created,
        ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        port: 0,
        task_token: "fake".to_string(),
        input_files: Vec::new(),
        output_files: Vec::new(),
        task_result_file_id: None,
    };
    let _ = add_task(&"fake_multi_task".to_owned(), &fake_multi_task);
}
