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

use std::collections::HashMap;
use std::prelude::v1::*;

use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{
    ExecutorType, FileCrypto, FunctionArguments, Storable, TeaclaveInputFile, TeaclaveOutputFile,
};

const STAGED_TASK_PREFIX: &str = "staged-"; // staged-task-uuid
pub const QUEUE_KEY: &str = "staged-task";

pub type FunctionInputFiles = HashMap<String, FunctionInputFile>;
pub type FunctionOutputFiles = HashMap<String, FunctionOutputFile>;

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionInputFile {
    pub url: Url,
    pub hash: String,
    pub crypto_info: FileCrypto,
}

impl FunctionInputFile {
    pub fn new(url: Url, hash: impl ToString, crypto_info: FileCrypto) -> Self {
        Self {
            url,
            hash: hash.to_string(),
            crypto_info,
        }
    }

    pub fn from_teaclave_input_file(file: &TeaclaveInputFile) -> Self {
        Self {
            url: file.url.to_owned(),
            hash: file.hash.to_owned(),
            crypto_info: file.crypto_info,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionOutputFile {
    pub url: Url,
    pub crypto_info: FileCrypto,
}

impl FunctionOutputFile {
    pub fn new(url: Url, crypto_info: FileCrypto) -> Self {
        Self { url, crypto_info }
    }

    pub fn from_teaclave_output_file(file: &TeaclaveOutputFile) -> Self {
        Self {
            url: file.url.to_owned(),
            crypto_info: file.crypto_info,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct StagedTask {
    pub task_id: Uuid,
    pub function_id: Uuid,
    pub function_name: String,
    pub function_payload: Vec<u8>,
    pub function_arguments: FunctionArguments,
    pub input_data: FunctionInputFiles,
    pub output_data: FunctionOutputFiles,
}

impl Storable for StagedTask {
    fn key_prefix() -> &'static str {
        STAGED_TASK_PREFIX
    }

    fn uuid(&self) -> Uuid {
        self.task_id
    }
}

impl StagedTask {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn task_id(self, task_id: Uuid) -> Self {
        Self { task_id, ..self }
    }

    pub fn function_id(self, function_id: Uuid) -> Self {
        Self {
            function_id,
            ..self
        }
    }

    pub fn function_name(self, function_name: impl ToString) -> Self {
        Self {
            function_name: function_name.to_string(),
            ..self
        }
    }

    pub fn function_payload(self, function_payload: Vec<u8>) -> Self {
        Self {
            function_payload,
            ..self
        }
    }

    pub fn function_arguments(self, function_arguments: FunctionArguments) -> Self {
        Self {
            function_arguments,
            ..self
        }
    }

    pub fn input_data(self, input_data: FunctionInputFiles) -> Self {
        Self { input_data, ..self }
    }

    pub fn output_data(self, output_data: FunctionOutputFiles) -> Self {
        Self {
            output_data,
            ..self
        }
    }

    pub fn get_queue_key() -> &'static str {
        QUEUE_KEY
    }

    pub fn executor_type(&self) -> ExecutorType {
        if self.function_payload.is_empty() {
            ExecutorType::Native
        } else {
            ExecutorType::Python
        }
    }
}
