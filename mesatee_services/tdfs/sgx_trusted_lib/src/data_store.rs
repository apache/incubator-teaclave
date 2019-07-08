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

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use lazy_static::lazy_static;
use mesatee_core::db::Memdb;
use std::format;

#[derive(Clone)]
pub struct FileMeta {
    pub user_id: String,
    pub file_name: String,
    pub sha256: String,
    pub file_size: u32,
    pub key_id: String,
    pub storage_path: String,
    pub task_id: Option<String>,
    pub allow_policy: u32, //0: owner, 1: owner & collaborator, 2: everyone
    pub collaborator_list: Vec<String>,
}

lazy_static! {
    pub static ref FILE_STORE: Memdb<String, FileMeta> = {
        let db = Memdb::<String, FileMeta>::open().expect("failed to open database");
        let fake_file_record = FileMeta {
            user_id: "fake_file_owner".to_string(),
            file_name: "fake_file".to_string(),
            sha256: "aaa".to_string(),
            file_size: 100,
            key_id: "fake_kms_record".to_string(),
            storage_path: "fake_file".to_string(),
            task_id: None,
            allow_policy: 0,
            collaborator_list: Vec::new(),
        };
        let _ = db.set(&"fake_file_record".to_string(), &fake_file_record);
        let fake_file_without_key = FileMeta {
            user_id: "fake_file_owner".to_string(),
            file_name: "fake_file".to_string(),
            sha256: "aaa".to_string(),
            file_size: 100,
            key_id: "kms_record_none".to_string(),
            storage_path: "fake_file".to_string(),
            task_id: None,
            allow_policy: 0,
            collaborator_list: Vec::new(),
        };
        let _ = db.set(&"fake_file_without_key".to_string(), &fake_file_without_key);

        db
    };
}

impl FileMeta {
    pub fn get_access_path(&self) -> String {
        format!("/tmp/{}", self.storage_path)
    }

    pub fn check_permission(&self, user_id: &str) -> bool {
        let file_owner = &self.user_id;
        let allow_policy = self.allow_policy;
        if (file_owner == user_id) || (allow_policy == 2) {
            return true;
        }
        if self.allow_policy == 1 {
            for collaborator_id in self.collaborator_list.iter() {
                if user_id == collaborator_id {
                    return true;
                }
            }
        }
        false
    }
}

pub fn verify_user(_user_id: &str, user_token: &str) -> bool {
    if user_token == "error_token" {
        return false;
    }
    true
}
