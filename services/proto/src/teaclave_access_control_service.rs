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

use crate::teaclave_access_control_service_proto as proto;
pub use proto::teaclave_access_control_client::TeaclaveAccessControlClient;
pub use proto::teaclave_access_control_server::{
    TeaclaveAccessControl, TeaclaveAccessControlServer,
};
pub use proto::*;

impl AuthorizeDataRequest {
    pub fn new(subject_user_id: impl Into<String>, object_data_id: impl Into<String>) -> Self {
        Self {
            subject_user_id: subject_user_id.into(),
            object_data_id: object_data_id.into(),
        }
    }
}

impl AuthorizeDataResponse {
    pub fn new(accept: bool) -> Self {
        Self { accept }
    }
}

impl AuthorizeFunctionRequest {
    pub fn new(subject_user_id: impl Into<String>, object_function_id: impl Into<String>) -> Self {
        Self {
            subject_user_id: subject_user_id.into(),
            object_function_id: object_function_id.into(),
        }
    }
}

impl AuthorizeFunctionResponse {
    pub fn new(accept: bool) -> Self {
        Self { accept }
    }
}

impl AuthorizeTaskRequest {
    pub fn new(subject_user_id: impl Into<String>, object_task_id: impl Into<String>) -> Self {
        Self {
            subject_user_id: subject_user_id.into(),
            object_task_id: object_task_id.into(),
        }
    }
}

impl AuthorizeTaskResponse {
    pub fn new(accept: bool) -> Self {
        Self { accept }
    }
}

impl AuthorizeStagedTaskResponse {
    pub fn new(accept: bool) -> Self {
        Self { accept }
    }
}
