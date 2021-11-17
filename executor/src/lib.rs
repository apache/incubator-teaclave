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
extern crate sgx_tstd as std;

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

#[macro_use]
extern crate log;

#[cfg(executor_builtin)]
mod builtin;
mod context;
#[cfg(executor_mesapy)]
mod mesapy;
#[cfg(executor_wamr)]
mod wamr;

#[cfg(executor_builtin)]
pub use builtin::BuiltinFunctionExecutor;
#[cfg(executor_mesapy)]
pub use mesapy::MesaPy;
#[cfg(executor_wamr)]
pub use wamr::WAMicroRuntime;

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;

    pub fn run_tests() -> bool {
        let mut v: Vec<bool> = Vec::new();
        v.push(context::tests::run_tests());
        #[cfg(executor_mesapy)]
        v.push(mesapy::tests::run_tests());
        #[cfg(executor_builtin)]
        v.push(builtin::tests::run_tests());
        #[cfg(executor_wamr)]
        v.push(wamr::tests::run_tests());
        v.iter().all(|&x| x)
    }
}
