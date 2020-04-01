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

use crate::{ExecutorType, Storable};
use serde::{Deserialize, Serialize};
use std::prelude::v1::*;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionInput {
    pub name: String,
    pub description: String,
}

impl FunctionInput {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionOutput {
    pub name: String,
    pub description: String,
}

impl FunctionOutput {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

const FUNCION_PREFIX: &str = "function";

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Function {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub public: bool,
    pub executor_type: ExecutorType,
    pub payload: Vec<u8>,
    pub arguments: Vec<String>,
    pub inputs: Vec<FunctionInput>,
    pub outputs: Vec<FunctionOutput>,
    pub owner: String,
}

impl Function {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(self, id: Uuid) -> Self {
        Self { id, ..self }
    }

    pub fn executor_type(self, executor_type: ExecutorType) -> Self {
        Self {
            executor_type,
            ..self
        }
    }

    pub fn name(self, name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            ..self
        }
    }

    pub fn description(self, description: impl ToString) -> Self {
        Self {
            description: description.to_string(),
            ..self
        }
    }

    pub fn payload(self, payload: Vec<u8>) -> Self {
        Self { payload, ..self }
    }

    pub fn public(self, public: bool) -> Self {
        Self { public, ..self }
    }

    pub fn arguments(self, arguments: Vec<String>) -> Self {
        Self { arguments, ..self }
    }

    pub fn inputs(self, inputs: Vec<FunctionInput>) -> Self {
        Self { inputs, ..self }
    }

    pub fn outputs(self, outputs: Vec<FunctionOutput>) -> Self {
        Self { outputs, ..self }
    }

    pub fn owner(self, owner: impl ToString) -> Self {
        Self {
            owner: owner.to_string(),
            ..self
        }
    }
}

impl Storable for Function {
    fn key_prefix() -> &'static str {
        FUNCION_PREFIX
    }

    fn uuid(&self) -> Uuid {
        self.id
    }
}
