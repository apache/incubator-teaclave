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

use std::collections::hash_map::{IntoIter, Iter, IterMut};
use std::collections::HashMap;
use std::prelude::v1::*;

use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{
    Executor, ExecutorType, FileAuthTag, FileCrypto, FunctionArguments, Storable,
    TeaclaveInputFile, TeaclaveOutputFile,
};

const STAGED_TASK_PREFIX: &str = "staged-"; // staged-task-uuid
pub const QUEUE_KEY: &str = "staged-task";

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct FunctionInputFiles {
    inner: HashMap<String, FunctionInputFile>,
}

impl FunctionInputFiles {
    pub fn new(entries: HashMap<String, FunctionInputFile>) -> Self {
        entries.into()
    }
    pub fn iter(&self) -> Iter<String, FunctionInputFile> {
        self.inner.iter()
    }
}

impl IntoIterator for FunctionInputFiles {
    type Item = (String, FunctionInputFile);
    type IntoIter = IntoIter<String, FunctionInputFile>;

    fn into_iter(self) -> IntoIter<String, FunctionInputFile> {
        self.inner.into_iter()
    }
}

impl std::convert::From<HashMap<String, FunctionInputFile>> for FunctionInputFiles {
    fn from(entries: HashMap<String, FunctionInputFile>) -> FunctionInputFiles {
        FunctionInputFiles { inner: entries }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct FunctionOutputFiles {
    inner: HashMap<String, FunctionOutputFile>,
}

impl IntoIterator for FunctionOutputFiles {
    type Item = (String, FunctionOutputFile);
    type IntoIter = IntoIter<String, FunctionOutputFile>;

    fn into_iter(self) -> IntoIter<String, FunctionOutputFile> {
        self.inner.into_iter()
    }
}

impl FunctionOutputFiles {
    pub fn new(entries: HashMap<String, FunctionOutputFile>) -> Self {
        entries.into()
    }

    pub fn iter(&self) -> Iter<String, FunctionOutputFile> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<String, FunctionOutputFile> {
        self.inner.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl std::convert::From<HashMap<String, FunctionOutputFile>> for FunctionOutputFiles {
    fn from(entries: HashMap<String, FunctionOutputFile>) -> FunctionOutputFiles {
        FunctionOutputFiles { inner: entries }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionInputFile {
    pub url: Url,
    pub cmac: FileAuthTag,
    pub crypto_info: FileCrypto,
}

impl FunctionInputFile {
    pub fn new(url: Url, cmac: FileAuthTag, crypto: impl Into<FileCrypto>) -> Self {
        Self {
            url,
            cmac,
            crypto_info: crypto.into(),
        }
    }

    pub fn from_teaclave_input_file(file: &TeaclaveInputFile) -> Self {
        Self {
            url: file.url.to_owned(),
            cmac: file.cmac.to_owned(),
            crypto_info: file.crypto_info,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionOutputFile {
    pub url: Url,
    pub crypto_info: FileCrypto,
}

impl FunctionOutputFile {
    pub fn new(url: Url, crypto: impl Into<FileCrypto>) -> Self {
        Self {
            url,
            crypto_info: crypto.into(),
        }
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
    pub executor: Executor,
    pub executor_type: ExecutorType,
    pub function_name: String,
    pub function_arguments: FunctionArguments,
    pub function_payload: Vec<u8>,
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

    pub fn executor(self, executor: Executor) -> Self {
        Self { executor, ..self }
    }

    pub fn function_name(self, name: impl ToString) -> Self {
        Self {
            function_name: name.to_string(),
            ..self
        }
    }

    pub fn function_arguments(self, function_arguments: impl Into<FunctionArguments>) -> Self {
        Self {
            function_arguments: function_arguments.into(),
            ..self
        }
    }

    pub fn function_payload(self, function_payload: Vec<u8>) -> Self {
        Self {
            function_payload,
            ..self
        }
    }

    pub fn input_data(self, input_data: impl Into<FunctionInputFiles>) -> Self {
        Self {
            input_data: input_data.into(),
            ..self
        }
    }

    pub fn output_data(self, output_data: impl Into<FunctionOutputFiles>) -> Self {
        Self {
            output_data: output_data.into(),
            ..self
        }
    }

    pub fn executor_type(self, executor_type: ExecutorType) -> Self {
        Self {
            executor_type,
            ..self
        }
    }

    pub fn get_queue_key() -> &'static str {
        QUEUE_KEY
    }
}
