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

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use std::io;

pub trait TeaclaveRuntime {
    fn open_input(&self, identifier: &str) -> anyhow::Result<Box<dyn io::Read>>;
    fn create_output(&self, identifier: &str) -> anyhow::Result<Box<dyn io::Write>>;
    // TODO: add more constrained capabilities
}

mod default;
pub use default::DefaultRuntime;

#[cfg(feature = "enclave_unit_test")]
mod raw_io;
#[cfg(feature = "enclave_unit_test")]
pub use raw_io::RawIoRuntime;

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use sgx_tunittest::*;

    pub fn run_tests() -> usize {
        rsgx_unit_tests!(
            //DefultRuntime::tests::test_open_input();
            //DefultRuntime::tests::test_create_output();
        )
    }
}
