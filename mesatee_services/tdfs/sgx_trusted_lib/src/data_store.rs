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

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use lazy_static::lazy_static;
use mesatee_core::db::Memdb;
use mesatee_core::{Error, ErrorKind, Result};
use std::collections::HashSet;
use tms_internal_proto::TaskInfo;
use std::env;
use std::path::Path;
use std::sync::SgxMutex;

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
    // At this moment, this is just a workaround;
    // Later the data structure will be redesigned according to the persistent db, lock mechanism and architecture.
    pub static ref USER_FILE_STORE: Memdb<String, HashSet<String>> = {
        Memdb::<String, HashSet<String>>::open().expect("cannot open db")
    };
    pub static ref UPDATELOCK: SgxMutex<u32> = SgxMutex::new(0);

    pub static ref FILE_STORE: Memdb<String, FileMeta> = {
        Memdb::<String, FileMeta>::open().expect("failed to open database")
    };
}

impl FileMeta {
    pub fn get_access_path(&self) -> String {
        let storage_dir = env::var("MESATEE_STORAGE_DIR").unwrap_or_else(|_| "/tmp".into());
        Path::new(&storage_dir)
            .join(&self.storage_path)
            .to_string_lossy()
            .to_string()
    }

    pub fn check_user_permission(&self, user_id: &str) -> bool {
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

// Before calling this function, use lock to avoid data race;
fn add_file_to_user(file_id: &str, user_id: &str) -> Result<()> {
    let uid = user_id.to_owned();
    let id_set = USER_FILE_STORE.get(&uid)?;
    match id_set {
        Some(mut set) => {
            set.insert(file_id.to_owned());
            USER_FILE_STORE.set(&uid, &set)?;
        }
        None => {
            let mut set = HashSet::<String>::new();
            set.insert(file_id.to_owned());
            USER_FILE_STORE.set(&uid, &set)?;
        }
    }
    Ok(())
}
pub fn add_file(file_id: &str, file_meta: &FileMeta) -> Result<()> {
    let _ = FILE_STORE.set(&file_id.to_owned(), &file_meta)?;
    let _lock = UPDATELOCK.lock()?;
    add_file_to_user(file_id, &file_meta.user_id)?;
    if file_meta.allow_policy == 1 {
        for collaborator in file_meta.collaborator_list.iter() {
            add_file_to_user(file_id, &collaborator)?;
        }
    }
    Ok(())
}

// Before calling this function, use lock to avoid data race;
fn del_file_for_user(file_id: &str, user_id: &str) -> Result<()> {
    let uid = user_id.to_owned();
    let id_set = USER_FILE_STORE.get(&uid)?;
    if let Some(mut set) = id_set {
        set.remove(&file_id.to_owned());
        USER_FILE_STORE.set(&uid, &set)?;
    }
    Ok(())
}
pub fn del_file(file_id: &str) -> Result<FileMeta> {
    let file_meta = FILE_STORE
        .del(&file_id.to_owned())?
        .ok_or_else(|| Error::from(ErrorKind::MissingValue))?;
    let _lock = UPDATELOCK.lock()?;
    del_file_for_user(file_id, &file_meta.user_id)?;
    if file_meta.allow_policy == 1 {
        for collaborator in file_meta.collaborator_list.iter() {
            del_file_for_user(file_id, &collaborator)?;
        }
    }
    Ok(file_meta)
}

// For API Test, called by enclave_init
pub fn add_test_infomation() {
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
    let _ = add_file(&"fake_file_record".to_string(), &fake_file_record);
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
    let _ = add_file(&"fake_file_without_key".to_string(), &fake_file_without_key);

    let mut fake_file_with_collaborator = FileMeta {
        user_id: "fake_file_owner".to_string(),
        file_name: "fake_file".to_string(),
        sha256: "aaa".to_string(),
        file_size: 100,
        key_id: "kms_record_none".to_string(),
        storage_path: "fake_file".to_string(),
        task_id: None,
        allow_policy: 1,
        collaborator_list: vec!["fake".to_string()],
    };

    let _ = add_file(
        &"fake_file_with_collaborator".to_string(),
        &fake_file_with_collaborator,
    );

    fake_file_with_collaborator.key_id = "fake_kms_record_to_be_deleted".to_string();
    let _ = add_file(
        &"fake_file_to_be_deleted".to_string(),
        &fake_file_with_collaborator,
    );
}

pub fn check_task_read_permission(task_info: &TaskInfo, file_id: &str) -> bool {
    // file is in the list of input files
    for task_file in task_info.input_files.iter() {
        if task_file.file_id == file_id {
            return true;
        }
    }

    // file belongs to the task creator
    let file_meta = match FILE_STORE.get(&file_id.to_owned()) {
        Ok(value) => value,
        Err(_) => return false,
    };
    let file_meta = match file_meta {
        Some(value) => value,
        None => return false,
    };
    file_meta.check_user_permission(&task_info.user_id)
}

pub fn check_task_write_permission(task_info: &TaskInfo, user_list: &[&str]) -> bool {
    let mut list: HashSet<&str> = HashSet::new();
    list.insert(&task_info.user_id);
    for collaborator in task_info.collaborator_list.iter() {
        list.insert(&collaborator.user_id);
    }
    for uid in user_list.iter() {
        if !list.contains(uid) {
            return false;
        }
    }
    true
}
