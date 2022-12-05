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

use crate::{ExecutorType, Storable, UserID};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionInput {
    pub name: String,
    pub description: String,
    pub optional: bool,
}

impl FunctionInput {
    pub fn new(name: impl Into<String>, description: impl Into<String>, optional: bool) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            optional,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionOutput {
    pub name: String,
    pub description: String,
    pub optional: bool,
}

impl FunctionOutput {
    pub fn new(name: impl Into<String>, description: impl Into<String>, optional: bool) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            optional,
        }
    }
}

const USER_PREFIX: &str = "user";

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: UserID,
    pub registered_functions: Vec<String>,
    pub allowed_functions: Vec<String>,
}

impl Storable for User {
    fn key_prefix() -> &'static str {
        USER_PREFIX
    }

    fn uuid(&self) -> Uuid {
        Uuid::new_v5(&Uuid::NAMESPACE_DNS, self.id.to_string().as_bytes())
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
    pub owner: UserID,
    pub user_allowlist: Vec<String>,
}

#[derive(Default)]
pub struct FunctionBuilder {
    function: Function,
}

impl FunctionBuilder {
    pub fn new() -> Self {
        Self {
            function: Function::default(),
        }
    }

    pub fn id(mut self, id: Uuid) -> Self {
        self.function.id = id;
        self
    }

    pub fn executor_type(mut self, executor_type: ExecutorType) -> Self {
        self.function.executor_type = executor_type;
        self
    }

    pub fn name(mut self, name: impl ToString) -> Self {
        self.function.name = name.to_string();
        self
    }

    pub fn description(mut self, description: impl ToString) -> Self {
        self.function.description = description.to_string();
        self
    }

    pub fn payload(mut self, payload: Vec<u8>) -> Self {
        self.function.payload = payload;
        self
    }

    pub fn public(mut self, public: bool) -> Self {
        self.function.public = public;
        self
    }

    pub fn arguments(mut self, arguments: Vec<String>) -> Self {
        self.function.arguments = arguments;
        self
    }

    pub fn inputs(mut self, inputs: Vec<FunctionInput>) -> Self {
        self.function.inputs = inputs;
        self
    }

    pub fn outputs(mut self, outputs: Vec<FunctionOutput>) -> Self {
        self.function.outputs = outputs;
        self
    }

    pub fn owner(mut self, owner: impl Into<UserID>) -> Self {
        self.function.owner = owner.into();
        self
    }

    pub fn user_allowlist(mut self, user_allowlist: Vec<String>) -> Self {
        self.function.user_allowlist = user_allowlist;
        self
    }

    pub fn build(self) -> Function {
        self.function
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
