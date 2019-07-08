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

use serde_derive::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InvokeTaskRequest {
    pub task_id: String,
    pub function_name: String,
    pub task_token: String,
    pub payload: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InvokeTaskResponse {
    pub result: String,
}

impl InvokeTaskRequest {
    pub fn new(
        task_id: &str,
        function_name: &str,
        task_token: &str,
        payload: Option<&str>,
    ) -> InvokeTaskRequest {
        InvokeTaskRequest {
            task_id: task_id.to_owned(),
            function_name: function_name.to_owned(),
            task_token: task_token.to_owned(),
            payload: payload.map(|s| s.to_owned()),
        }
    }
}

impl InvokeTaskResponse {
    pub fn new(result: &str) -> InvokeTaskResponse {
        InvokeTaskResponse {
            result: result.to_owned(),
        }
    }
}
