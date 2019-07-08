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

use fns_proto::InvokeTaskRequest;
use mesatee_core::Result;

#[derive(Clone, Debug)]
pub struct WorkerInput {
    pub function_name: String,
    pub input_files: Vec<String>,
    pub payload: Option<String>,
}

// Execute the task
pub trait TaskExecutor: Sized {
    fn init(request: &InvokeTaskRequest) -> Result<Self>;
    fn execute(&mut self) -> Result<String>;
    fn finalize(self) -> Result<()>;
}

// Help worker to access resouces or interact with outside
pub trait WorkerHelper {
    fn read_file(&mut self, file_id: &str) -> Result<Vec<u8>>;
    fn save_file_for_task_creator(&mut self, data: &[u8]) -> Result<String>;
    fn save_file_for_all_participants(&mut self, data: &[u8]) -> Result<String>;
    fn save_file_for_file_owner(&mut self, data: &[u8], file_id: &str) -> Result<String>;
    fn get_input(&mut self) -> WorkerInput;
}

// A worker to do the computation. It can be a module/enclave.
pub trait Worker {
    fn launch(&mut self) -> Result<()>;
    fn compute(&mut self, helper: &mut WorkerHelper) -> Result<String>;
    fn finalize(&mut self) -> Result<()>;
}
