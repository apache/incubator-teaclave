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

use teaclave_types::FunctionArguments;
use teaclave_types::{TeaclaveFunction, TeaclaveRuntime};

#[derive(Default)]
pub struct Echo;

impl TeaclaveFunction for Echo {
    fn execute(
        &self,
        _runtime: Box<dyn TeaclaveRuntime + Send + Sync>,
        arguments: FunctionArguments,
    ) -> anyhow::Result<String> {
        let message = arguments.get("message")?.to_string();
        Ok(message)
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use teaclave_runtime::*;
    use teaclave_test_utils::*;
    use teaclave_types::*;

    pub fn run_tests() -> bool {
        run_tests!(test_echo)
    }

    fn test_echo() {
        let func_args = FunctionArguments::new(hashmap!(
            "message"  => "Hello Teaclave!"
        ));

        let input_files = StagedFiles::default();
        let output_files = StagedFiles::default();

        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));
        let function = Echo;

        let summary = function.execute(runtime, func_args).unwrap();
        assert_eq!(summary, "Hello Teaclave!");
    }
}
