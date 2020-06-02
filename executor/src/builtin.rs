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

use teaclave_function::{
    Echo, GbdtPredict, GbdtTrain, LogisticRegressionPredict, LogisticRegressionTrain, OnlineDecrypt,
};
use teaclave_types::{FunctionArguments, FunctionRuntime, TeaclaveExecutor};

use anyhow::{bail, Result};

#[derive(Default)]
pub struct BuiltinFunctionExecutor;

impl TeaclaveExecutor for BuiltinFunctionExecutor {
    fn execute(
        &self,
        name: String,
        arguments: FunctionArguments,
        _payload: String,
        runtime: FunctionRuntime,
    ) -> Result<String> {
        match name.as_str() {
            #[cfg(feature = "builtin_echo")]
            Echo::NAME => Echo::new().run(arguments, runtime),
            #[cfg(feature = "builtin_gbdt_predict")]
            GbdtPredict::NAME => GbdtPredict::new().run(arguments, runtime),
            #[cfg(feature = "builtin_gbdt_train")]
            GbdtTrain::NAME => GbdtTrain::new().run(arguments, runtime),
            #[cfg(feature = "builtin_logistic_regression_train")]
            LogisticRegressionTrain::NAME => LogisticRegressionTrain::new().run(arguments, runtime),
            #[cfg(feature = "builtin_logistic_regression_predict")]
            LogisticRegressionPredict::NAME => {
                LogisticRegressionPredict::new().run(arguments, runtime)
            }
            #[cfg(feature = "builtin_online_decrypt")]
            OnlineDecrypt::NAME => OnlineDecrypt::new().run(arguments, runtime),
            _ => bail!("Function not found."),
        }
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    pub fn run_tests() -> bool {
        true
    }
}
