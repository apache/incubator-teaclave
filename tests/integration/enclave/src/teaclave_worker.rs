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

use std::prelude::v1::*;

use serde_json::json;
use teaclave_crypto::TeaclaveFile128Key;
use teaclave_types::{
    hashmap, read_all_bytes, Executor, ExecutorType, FileAuthTag, FunctionArguments,
    StagedFileInfo, StagedFiles, StagedFunctionBuilder,
};
use teaclave_worker::Worker;

fn test_start_worker() {
    let arguments = FunctionArguments::from_json(json!({
        "feature_size": 4,
        "max_depth": 4,
        "iterations": 100,
        "shrinkage": 0.1,
        "feature_sample_ratio": 1.0,
        "data_sample_ratio": 1.0,
        "min_leaf_size": 1,
        "loss": "LAD",
        "training_optimization_level": 2
    }))
    .unwrap();

    let plain_input = "fixtures/functions/gbdt_training/train.txt";
    let enc_output = "fixtures/functions/gbdt_training/model.enc.out";
    let expected_output = "fixtures/functions/gbdt_training/expected_model.txt";

    let input_info = StagedFileInfo::create_with_plaintext_file(plain_input).unwrap();
    let input_files = StagedFiles::new(hashmap!(
        "training_data" => input_info));

    let output_info = StagedFileInfo::new(
        enc_output,
        TeaclaveFile128Key::random(),
        FileAuthTag::mock(),
    );

    let output_files = StagedFiles::new(hashmap!(
        "trained_model" => output_info.clone()));

    let staged_function = StagedFunctionBuilder::new()
        .executor_type(ExecutorType::Builtin)
        .executor(Executor::Builtin)
        .name("builtin-gbdt-train")
        .arguments(arguments)
        .input_files(input_files)
        .output_files(output_files)
        .runtime_name("default")
        .build();

    let worker = Worker::default();

    let summary = worker.invoke_function(staged_function).unwrap();
    assert_eq!(summary, "Trained 120 lines of data.");

    let result = output_info.get_plaintext().unwrap();
    let expected = read_all_bytes(expected_output).unwrap();
    assert_eq!(&result[..], &expected[..]);
}

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    run_tests!(test_start_worker)
}
