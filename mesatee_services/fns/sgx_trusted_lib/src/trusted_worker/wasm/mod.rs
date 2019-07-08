// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::trait_defs::{WorkerHelper, WorkerInput};
use mesatee_core::{Error, ErrorKind, Result};
use serde_derive::Serialize;
use serde_json;

mod sgxwasm;
mod sgxwasm_compute;

use sgxwasm::BoundaryValue;

#[derive(Serialize)]
struct FaasInterpreterError;

pub fn wasmi_from_buffer(_helper: &mut WorkerHelper, input: WorkerInput) -> Result<String> {
    let payload = match input.payload {
        Some(value) => value,
        None => return Err(Error::from(ErrorKind::MissingValue)),
    };

    let action_list: Vec<sgxwasm::SgxWasmAction> = serde_json::from_str(&payload)
        .or_else(|_| Err(Error::from(ErrorKind::InvalidInputError)))?;
    let mut result: Vec<std::result::Result<Option<BoundaryValue>, FaasInterpreterError>> =
        Vec::new();
    let mut driver = sgxwasm_compute::sgxwasm_init();

    for action_req in action_list.into_iter() {
        let one_result = sgxwasm_compute::sgxwasm_run_action(&mut driver, action_req);
        result.push(one_result.map_err(|_| FaasInterpreterError));
    }

    let serialized_result = serde_json::to_string(&result)
        .or_else(|_| Err(Error::from(ErrorKind::OutputGenerationError)))?;
    Ok(serialized_result)
}
