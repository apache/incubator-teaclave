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

#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
#[macro_use]
extern crate sgx_tstd as std;

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

mod attestation;
mod crypto;
mod error;
mod file;
mod file_agent;
mod function;
mod macros;
mod staged_file;
mod staged_function;
mod staged_task;
mod storage;
mod task;
mod task_state;
mod worker;

pub use attestation::*;
pub use crypto::*;
pub use error::*;
pub use file::*;
pub use file_agent::*;
pub use function::*;
pub use macros::*;
pub use staged_file::*;
pub use staged_function::*;
pub use staged_task::*;
pub use storage::*;
pub use task::*;
pub use task_state::*;
pub use worker::*;

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;

    pub fn run_tests() -> bool {
        worker::tests::run_tests()
    }
}
