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

use crate::worker::{FunctionType, Worker, WorkerContext};
use mesatee_core::{Error, ErrorKind, Result};

mod basic;
mod compute;
use compute::SetIntersection;

pub struct PSIWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<PSIWorkerInput>,
}
impl PSIWorker {
    pub fn new() -> Self {
        PSIWorker {
            worker_id: 0,
            func_name: "psi".to_string(),
            func_type: FunctionType::Multiparty,
            input: None,
        }
    }
}

struct PSIWorkerInput {
    file1: String,
    file2: String,
}

impl Worker for PSIWorker {
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
        _dynamic_input: Option<String>,
        file_ids: Vec<String>,
    ) -> Result<()> {
        if file_ids.len() != 2 {
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
        self.input = Some(PSIWorkerInput {
            file1: file_ids[0].to_string(),
            file2: file_ids[1].to_string(),
        });
        Ok(())
    }
    fn execute(&mut self, context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        let file1 = &input.file1;
        let file2 = &input.file2;

        let plaintext1 = context.read_file(&file1)?;
        let plaintext2 = context.read_file(&file2)?;

        let mut si = SetIntersection::new();
        if !si.psi_add_hash_data(plaintext1, 0) {
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
        if !si.psi_add_hash_data(plaintext2, 1) {
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
        si.compute();
        let _result_file1 = context.save_file_for_file_owner(&si.data[0].result, file1)?;
        let _result_file2 = context.save_file_for_file_owner(&si.data[1].result, file2)?;

        Ok("Finished".to_string())
    }
}
