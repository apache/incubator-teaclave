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

mod echo;
mod gbdt_predict;
mod gbdt_train;
mod logistic_regression_predict;
mod logistic_regression_train;
mod online_decrypt;

pub use echo::Echo;
pub use gbdt_predict::GbdtPredict;
pub use gbdt_train::GbdtTrain;
pub use logistic_regression_predict::LogisticRegressionPredict;
pub use logistic_regression_train::LogisticRegressionTrain;
pub use online_decrypt::OnlineDecrypt;

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use teaclave_test_utils::check_all_passed;

    pub fn run_tests() -> bool {
        check_all_passed!(
            echo::tests::run_tests(),
            gbdt_train::tests::run_tests(),
            gbdt_predict::tests::run_tests(),
            logistic_regression_train::tests::run_tests(),
            logistic_regression_predict::tests::run_tests(),
            online_decrypt::tests::run_tests(),
        )
    }
}
