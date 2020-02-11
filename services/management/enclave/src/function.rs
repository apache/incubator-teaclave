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
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::prelude::v1::*;
use teaclave_proto::teaclave_frontend_service::{
    FunctionInput, FunctionOutput, RegisterFunctionRequest,
};

use uuid::Uuid;
const SCRIPT_PREFIX: &str = "function-";
const NATIVE_PREFIX: &str = "native-";
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Function {
    pub(crate) function_id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) payload: Vec<u8>,
    pub(crate) is_public: bool,
    pub(crate) arg_list: Vec<String>,
    pub(crate) input_list: Vec<FunctionInput>,
    pub(crate) output_list: Vec<FunctionOutput>,
    pub(crate) owner: String,
    pub(crate) is_native: bool,
}

impl Function {
    pub(crate) fn new_from_register_request(
        request: RegisterFunctionRequest,
        owner: String,
    ) -> Function {
        let function_id = format!("{}{}", SCRIPT_PREFIX, Uuid::new_v4().to_string());
        Function {
            function_id,
            name: request.name,
            description: request.description,
            payload: request.payload,
            is_public: request.is_public,
            arg_list: request.arg_list,
            input_list: request.input_list,
            output_list: request.output_list,
            owner,
            is_native: false,
        }
    }

    pub(crate) fn from_slice(bytes: &[u8]) -> Result<Self> {
        let ret: Function =
            serde_json::from_slice(&bytes).map_err(|_| anyhow!("failed to Deserialize"))?;
        Ok(ret)
    }

    pub(crate) fn to_vec(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self).map_err(|_| anyhow!("failed to Serialize"))
    }

    pub(crate) fn get_key_vec(&self) -> Vec<u8> {
        self.function_id.as_bytes().to_vec()
    }

    pub(crate) fn is_function_id(id: &str) -> bool {
        id.starts_with(NATIVE_PREFIX) || id.starts_with(SCRIPT_PREFIX)
    }
}
