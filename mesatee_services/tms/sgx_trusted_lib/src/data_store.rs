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

use mesatee_core::db::Memdb;
use mesatee_core::Result;
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
    pub static ref TASK_STORE: Memdb<String, TaskInfo> = {
        let store = Memdb::<String, TaskInfo>::open().expect("cannot open db");
        // For API Test
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
        let _ = store.set(&"fake".to_string(), &fake_task);

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
        let _ = store.set(&"fake_multi_task".to_string(), &fake_multi_task);

        store
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
