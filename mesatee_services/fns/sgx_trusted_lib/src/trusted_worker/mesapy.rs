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
use itertools::Itertools;
use mesatee_core::{Error, ErrorKind, Result};
use std::format;

const MAXPYBUFLEN: usize = 1024;
const MESAPY_ERROR_BUFFER_TOO_SHORT: i64 = -1i64;
const MESAPY_EXEC_ERROR: i64 = -2i64;

extern "C" {
    fn mesapy_exec(input: *const u8, output: *mut u8, buflen: u64) -> i64;
}

pub struct MesaPyWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<MesaPyWorkerWorkerInput>,
}
struct MesaPyWorkerWorkerInput {
    py_script_vec: Vec<u8>,
}
impl MesaPyWorker {
    pub fn new() -> Self {
        MesaPyWorker {
            worker_id: 0,
            func_name: "mesapy_from_buffer".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}

impl Worker for MesaPyWorker {
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

        let mut py_script_vec =
            base64::decode(&payload).or_else(|_| Err(Error::from(ErrorKind::InvalidInputError)))?;
        py_script_vec.push(0u8);
        self.input = Some(MesaPyWorkerWorkerInput { py_script_vec });
        Ok(())
    }

    fn execute(&mut self, _context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        let mut py_result = [0u8; MAXPYBUFLEN];

        let result = unsafe {
            mesapy_exec(
                input.py_script_vec.as_ptr(),
                &mut py_result as *mut _ as *mut u8,
                MAXPYBUFLEN as u64,
            )
        };

        match result {
            MESAPY_ERROR_BUFFER_TOO_SHORT => Ok("MESAPY_ERROR_BUFFER_TOO_SHORT".to_string()),
            MESAPY_EXEC_ERROR => Ok("MESAPY_EXEC_ERROR".to_string()),
            len => {
                let r: Vec<u8> = py_result.iter().take(len as usize).copied().collect();
                let payload = format!("marshal.loads(b\"\\x{:02X}\")", r.iter().format("\\x"));
                Ok(payload)
            }
        }
    }
}
