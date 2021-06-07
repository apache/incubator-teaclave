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

use crate::{Executor, ExecutorType, StagedFiles, TeaclaveRuntime};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::prelude::v1::*;

use anyhow::{Context, Result};

pub type FunctionRuntime = Box<dyn TeaclaveRuntime + Send + Sync>;
type ArgumentValue = serde_json::Value;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct FunctionArguments {
    #[serde(flatten)]
    inner: serde_json::Map<String, ArgumentValue>,
}

impl From<HashMap<String, String>> for FunctionArguments {
    fn from(map: HashMap<String, String>) -> Self {
        let inner = map.iter().fold(serde_json::Map::new(), |mut acc, (k, v)| {
            acc.insert(k.to_owned(), v.to_owned().into());
            acc
        });

        Self { inner }
    }
}

impl std::convert::TryFrom<String> for FunctionArguments {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let v: ArgumentValue = serde_json::from_str(&s)?;
        let inner = match v {
            ArgumentValue::Object(o) => o,
            _ => anyhow::bail!("Cannot convert to function arguments"),
        };

        Ok(Self { inner })
    }
}

impl FunctionArguments {
    pub fn from_json(json: ArgumentValue) -> Result<Self> {
        let inner = match json {
            ArgumentValue::Object(o) => o,
            _ => anyhow::bail!("Not an json object"),
        };

        Ok(Self { inner })
    }

    pub fn from_map(map: HashMap<String, String>) -> Self {
        map.into()
    }

    pub fn inner(&self) -> &serde_json::Map<String, ArgumentValue> {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut serde_json::Map<String, ArgumentValue> {
        &mut self.inner
    }

    pub fn get(&self, key: &str) -> anyhow::Result<&ArgumentValue> {
        self.inner
            .get(key)
            .with_context(|| format!("key not found: {}", key))
    }

    pub fn into_vec(self) -> Vec<String> {
        let mut vector = Vec::new();

        self.inner.into_iter().for_each(|(k, v)| {
            vector.push(k);
            match v {
                ArgumentValue::String(s) => vector.push(s),
                _ => vector.push(v.to_string()),
            }
        });

        vector
    }

    pub fn into_string(self) -> String {
        ArgumentValue::Object(self.inner).to_string()
    }
}

#[derive(Debug, Default)]
pub struct StagedFunction {
    pub name: String,
    pub arguments: FunctionArguments,
    pub payload: Vec<u8>,
    pub input_files: StagedFiles,
    pub output_files: StagedFiles,
    pub executor_type: ExecutorType,
    pub executor: Executor,
    pub runtime_name: String,
}

impl StagedFunction {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(self, name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            ..self
        }
    }

    pub fn executor(self, executor: Executor) -> Self {
        Self { executor, ..self }
    }

    pub fn payload(self, payload: Vec<u8>) -> Self {
        Self { payload, ..self }
    }

    pub fn arguments(self, arguments: FunctionArguments) -> Self {
        Self { arguments, ..self }
    }

    pub fn input_files(self, input_files: StagedFiles) -> Self {
        Self {
            input_files,
            ..self
        }
    }

    pub fn output_files(self, output_files: StagedFiles) -> Self {
        Self {
            output_files,
            ..self
        }
    }

    pub fn runtime_name(self, runtime_name: impl ToString) -> Self {
        Self {
            runtime_name: runtime_name.to_string(),
            ..self
        }
    }

    pub fn executor_type(self, executor_type: ExecutorType) -> Self {
        Self {
            executor_type,
            ..self
        }
    }
}
