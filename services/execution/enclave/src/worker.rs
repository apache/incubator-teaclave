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

use std::collections::HashMap;
use std::format;

use anyhow;
use serde_json;

use teaclave_proto::teaclave_execution_service::StagedFunctionExecuteRequest;
use teaclave_proto::teaclave_execution_service::TeaclaveExecutorSelector;
use teaclave_proto::teaclave_execution_service::TeaclaveWorkerFileInfo;

use crate::function::{self, FunctionArguments, TeaclaveFunction};
use crate::runtime::{self, TeaclaveRuntime};

pub struct Worker {
    runtimes: HashMap<String, RuntimeBuilder>,
    functions: HashMap<String, FunctionBuilder>,
}

impl Worker {
    pub fn new() -> Worker {
        Worker {
            functions: setup_functions(),
            runtimes: setup_runtimes(),
        }
    }

    pub fn invoke_function(&self, req: &StagedFunctionExecuteRequest) -> anyhow::Result<String> {
        let function = self.get_function(&req.executor_type, &req.function_name)?;
        let runtime = self.get_runtime(
            &req.runtime_name,
            req.input_files.clone(),
            req.output_files.clone(),
        )?;
        let unified_args = prepare_arguments(req)?;

        function.execute(runtime, unified_args)
    }

    fn get_runtime(
        &self,
        name: &str,
        input_files: HashMap<String, TeaclaveWorkerFileInfo>,
        output_files: HashMap<String, TeaclaveWorkerFileInfo>,
    ) -> anyhow::Result<Box<dyn TeaclaveRuntime + Send + Sync>> {
        let build_runtime = self
            .runtimes
            .get(name)
            .ok_or_else(|| anyhow::anyhow!(format!("Runtime {} not available.", name)))?;

        let runtime = build_runtime(input_files, output_files);
        Ok(runtime)
    }

    fn get_function(
        &self,
        func_type: &TeaclaveExecutorSelector,
        func_name: &str,
    ) -> anyhow::Result<Box<dyn TeaclaveFunction + Send + Sync>> {
        let identifier = make_function_identifier(func_type, func_name);
        let build_function = self
            .functions
            .get(&identifier)
            .ok_or_else(|| anyhow::anyhow!(format!("function not available: {}", identifier)))?;

        let function = build_function();
        Ok(function)
    }
}

fn make_function_identifier(func_type: &TeaclaveExecutorSelector, func_name: &str) -> String {
    let type_str = func_type.to_string();
    format!("{}-{}", type_str, func_name)
}

fn setup_functions() -> HashMap<String, FunctionBuilder> {
    let mut functions: HashMap<String, FunctionBuilder> = HashMap::new();
    functions.insert(
        make_function_identifier(&TeaclaveExecutorSelector::Native, "gbdt_training"),
        Box::new(|| Box::new(function::GbdtTraining::default())),
    );
    functions.insert(
        make_function_identifier(&TeaclaveExecutorSelector::Python, "mesapy"),
        Box::new(|| Box::new(function::Mesapy::default())),
    );
    functions
}

fn setup_runtimes() -> HashMap<String, RuntimeBuilder> {
    let mut runtimes: HashMap<String, RuntimeBuilder> = HashMap::new();
    runtimes.insert(
        "default".to_string(),
        Box::new(|input_files, output_files| {
            Box::new(runtime::DefaultRuntime::new(input_files, output_files))
        }),
    );

    runtimes
}

// Native functions (TeaclaveExecutorSelector::Native) are not allowed to have function payload.
// Script engines like Mesapy (TeaclaveExecutorSelector::Python) must have script payload.
// We assume that the script engines would take the script payload and
// script arguments from the wrapped argument.
fn prepare_arguments(req: &StagedFunctionExecuteRequest) -> anyhow::Result<FunctionArguments> {
    let unified_args = match &req.executor_type {
        TeaclaveExecutorSelector::Native => {
            assert!(req.function_payload.len() == 0);
            FunctionArguments(req.function_args.clone())
        }
        TeaclaveExecutorSelector::Python => {
            assert!(req.function_payload.len() > 0);
            let mut wrap_args = HashMap::new();
            let req_args = serde_json::to_string(&req.function_args)?;
            wrap_args.insert("py_payload".to_string(), req.function_payload.clone());
            wrap_args.insert("py_args".to_string(), req_args);
            FunctionArguments(wrap_args)
        }
    };

    Ok(unified_args)
}

type FunctionBuilder = Box<dyn Fn() -> Box<dyn TeaclaveFunction + Send + Sync> + Send + Sync>;
type RuntimeBuilder = Box<
    dyn Fn(
            HashMap<String, TeaclaveWorkerFileInfo>,
            HashMap<String, TeaclaveWorkerFileInfo>,
        ) -> Box<dyn TeaclaveRuntime + Send + Sync>
        + Send
        + Sync,
>;

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use std::untrusted::fs;

    pub fn test_start_worker() {
        let request_payload = r#"{
            "runtime_name": "default",
            "executor_type": "native",
            "function_name": "gbdt_training",
            "function_payload": "",
            "function_args": {
                "feature_size": "4",
                "max_depth": "4",
                "iterations": "100",
                "shrinkage": "0.1",
                "feature_sample_ratio": "1.0",
                "data_sample_ratio": "1.0",
                "min_leaf_size": "1",
                "loss": "LAD",
                "training_optimization_level": "2"
            },
            "input_files": {
                "training_data": {
                    "path": "test_cases/gbdt_training/train.txt",
                    "crypto_info": {
                        "aes_gcm128": {
                            "key": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
                            "iv": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
                        }
                    }
                }
            },
            "output_files": {
                "trained_model": {
                    "path": "test_cases/gbdt_training/model.txt.out",
                    "crypto_info": {
                        "aes_gcm128": {
                            "key": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
                            "iv": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
                        }
                    }
                }
            }
        }"#;


        let plain_output = "test_cases/gbdt_training/model.txt.out";
        let expected_output = "test_cases/gbdt_training/expected_model.txt";
        let request: StagedFunctionExecuteRequest =
            serde_json::from_str(request_payload).unwrap();
        let worker = Worker::new();
        let summary = worker.invoke_function(&request).unwrap();
        assert_eq!(summary, "Trained 120 lines of data.");

        let result = fs::read_to_string(&plain_output).unwrap();
        let expected = fs::read_to_string(&expected_output).unwrap();
        assert_eq!(&result[..], &expected[..]);
    }
}
