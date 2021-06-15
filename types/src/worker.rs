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

use crate::{FunctionArguments, FunctionRuntime, OutputsTags};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::convert::TryInto;
use std::io;
use std::prelude::v1::*;

pub trait TeaclaveRuntime {
    fn open_input(&self, identifier: &str) -> anyhow::Result<Box<dyn io::Read>>;
    fn create_output(&self, identifier: &str) -> anyhow::Result<Box<dyn io::Write>>;
}

pub trait TeaclaveExecutor {
    fn execute(
        &self,
        name: String,
        arguments: FunctionArguments,
        payload: Vec<u8>,
        runtime: FunctionRuntime,
    ) -> anyhow::Result<String>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ExecutorType {
    Builtin,
    Python,
    WAMicroRuntime,
}

impl std::default::Default for ExecutorType {
    fn default() -> Self {
        ExecutorType::Builtin
    }
}

impl std::convert::TryFrom<&str> for ExecutorType {
    type Error = anyhow::Error;

    fn try_from(selector: &str) -> anyhow::Result<Self> {
        let executor_type = match selector {
            "python" => ExecutorType::Python,
            "builtin" => ExecutorType::Builtin,
            "wamr" => ExecutorType::WAMicroRuntime,
            _ => anyhow::bail!("Invalid executor type: {}", selector),
        };
        Ok(executor_type)
    }
}

impl std::convert::TryFrom<String> for ExecutorType {
    type Error = anyhow::Error;

    fn try_from(selector: String) -> anyhow::Result<Self> {
        selector.as_str().try_into()
    }
}

impl std::convert::From<ExecutorType> for String {
    fn from(executor_type: ExecutorType) -> String {
        format!("{}", executor_type)
    }
}

impl std::fmt::Display for ExecutorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExecutorType::Builtin => write!(f, "builtin"),
            ExecutorType::Python => write!(f, "python"),
            ExecutorType::WAMicroRuntime => write!(f, "wamr"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Executor {
    MesaPy,
    Builtin,
    WAMicroRuntime,
}

impl std::default::Default for Executor {
    fn default() -> Self {
        Executor::MesaPy
    }
}

impl std::convert::TryFrom<&str> for Executor {
    type Error = anyhow::Error;

    fn try_from(selector: &str) -> anyhow::Result<Self> {
        let executor = match selector {
            "mesapy" => Executor::MesaPy,
            "builtin" => Executor::Builtin,
            "wamr" => Executor::WAMicroRuntime,
            _ => anyhow::bail!("Unsupported executor: {}", selector),
        };
        Ok(executor)
    }
}

impl std::convert::TryFrom<String> for Executor {
    type Error = anyhow::Error;

    fn try_from(selector: String) -> anyhow::Result<Self> {
        selector.as_str().try_into()
    }
}

impl std::fmt::Display for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Executor::MesaPy => write!(f, "mesapy"),
            Executor::Builtin => write!(f, "builtin"),
            Executor::WAMicroRuntime => write!(f, "wamr"),
        }
    }
}

#[derive(Debug)]
pub struct WorkerCapability {
    pub runtimes: HashSet<String>,
    pub executors: HashSet<String>,
}

#[derive(Debug, Default)]
pub struct ExecutionResult {
    pub return_value: Vec<u8>,
    pub tags_map: OutputsTags,
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;

    pub fn run_tests() -> bool {
        true
    }
}
