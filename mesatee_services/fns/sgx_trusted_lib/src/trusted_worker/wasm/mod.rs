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

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::worker::{FunctionType, Worker, WorkerContext};
use mesatee_core::{Error, ErrorKind, Result};
use serde_derive::Serialize;
use serde_json;

mod sgxwasm;
mod sgxwasm_compute;

use sgxwasm::BoundaryValue;

#[derive(Serialize)]
struct FaasInterpreterError;

pub struct WASMWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<WASMWorkerInput>,
}
struct WASMWorkerInput {
    action_list: Vec<sgxwasm::SgxWasmAction>,
}
impl WASMWorker {
    pub fn new() -> Self {
        WASMWorker {
            worker_id: 0,
            func_name: "wasmi_from_buffer".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}
impl Worker for WASMWorker {
    fn function_name(&self) -> &str {
        self.func_name.as_str()
    }
    fn function_type(&self) -> FunctionType {
        self.func_type
    }
    fn set_id(&mut self, worker_id: u32) {
        self.worker_id = worker_id;
    }
    fn id(&self) -> u32 {
        self.worker_id
    }
    fn prepare_input(
        &mut self,
        dynamic_input: Option<String>,
        _file_ids: Vec<String>,
    ) -> Result<()> {
        let payload = match dynamic_input {
            Some(value) => value,
            None => return Err(Error::from(ErrorKind::InvalidInputError)),
        };
        let action_list: Vec<sgxwasm::SgxWasmAction> = serde_json::from_str(&payload)
            .or_else(|_| Err(Error::from(ErrorKind::InvalidInputError)))?;
        self.input = Some(WASMWorkerInput { action_list });
        Ok(())
    }

    fn execute(&mut self, _context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        let mut result: Vec<std::result::Result<Option<BoundaryValue>, FaasInterpreterError>> =
            Vec::new();
        let mut driver = sgxwasm_compute::sgxwasm_init();

        for action_req in input.action_list.into_iter() {
            let one_result = sgxwasm_compute::sgxwasm_run_action(&mut driver, action_req);
            result.push(one_result.map_err(|_| FaasInterpreterError));
        }

        let serialized_result = serde_json::to_string(&result)
            .or_else(|_| Err(Error::from(ErrorKind::OutputGenerationError)))?;
        Ok(serialized_result)
    }
}
