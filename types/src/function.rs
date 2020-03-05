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

use crate::Storable;
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
#[derive(Debug, Deserialize, Serialize)]
pub struct Function {
    pub function_id: Uuid,
    pub name: String,
    pub description: String,
    pub payload: Vec<u8>,
    pub is_public: bool,
    pub arg_list: Vec<String>,
    pub input_list: Vec<FunctionInput>,
    pub output_list: Vec<FunctionOutput>,
    pub owner: String,
    pub is_native: bool,
}

impl Storable for Function {
    fn key_prefix() -> &'static str {
        FUNCION_PREFIX
    }

    fn uuid(&self) -> Uuid {
        self.function_id
    }
}
