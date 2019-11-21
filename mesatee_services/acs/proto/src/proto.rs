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

use std::collections::HashSet;

use serde_derive::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ACSRequest {
    Enforce(EnforceRequest),
    Announce(AnnounceRequest),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ACSResponse {
    Enforce(bool),
    Announce,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum EnforceRequest {
    // launch_task = task, participants
    LaunchTask(String, HashSet<String>),

    // access_data = task, data
    AccessData(String, String),

    // delete_data = usr, data
    DeleteData(String, String),

    // access_script = task, script
    AccessScript(String, String),

    // delete_script = usr, script
    DeleteScript(String, String),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AnnounceRequest {
    pub facts: Vec<AccessControlTerms>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum AccessControlTerms {
    // task_creator = task, usr
    TaskCreator(String, String),

    // task_participant = task, usr
    TaskParticipant(String, String),

    // data_owner = data, usr
    DataOwner(String, String),

    // script_owner = script, usr
    ScriptOwner(String, String),

    // is_public_script = script
    IsPublicScript(String),
}
