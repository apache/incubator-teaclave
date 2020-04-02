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

use rusty_machine::learning::logistic_reg::LogisticRegressor;
use rusty_machine::learning::optim::grad_desc::GradientDesc;
use rusty_machine::learning::SupModel;
use rusty_machine::linalg;
use std::format;
use std::io::{self, BufRead, BufReader, Write};

use teaclave_types::FunctionArguments;
use teaclave_types::{TeaclaveFunction, TeaclaveRuntime};

#[derive(Default)]
pub struct LogitRegTraining;

static TRAINING_DATA: &str = "training_data";
static OUT_MODEL_FILE: &str = "model_file";

impl TeaclaveFunction for LogitRegTraining {
    fn execute(
        &self,
        runtime: Box<dyn TeaclaveRuntime + Send + Sync>,
        arguments: FunctionArguments,
    ) -> anyhow::Result<String> {
        let alg_alpha = arguments.get("alg_alpha")?.as_f64()?;
        let alg_iters = arguments.get("alg_iters")?.as_usize()?;
        let feature_size = arguments.get("feature_size")?.as_usize()?;

        let input = runtime.open_input(TRAINING_DATA)?;
        let (flattend_features, targets) = parse_training_data(input, feature_size)?;
        let data_size = targets.len();
        let data_matrix = linalg::Matrix::new(data_size, feature_size, flattend_features);
        let targets = linalg::Vector::new(targets);

        let gd = GradientDesc::new(alg_alpha, alg_iters);
        let mut lr = LogisticRegressor::new(gd);
        lr.train(&data_matrix, &targets)?;

        let model_json = serde_json::to_string(&lr).unwrap();
        let mut model_file = runtime.create_output(OUT_MODEL_FILE)?;
        model_file.write_all(model_json.as_bytes())?;

        Ok(format!("Trained {} lines of data.", data_size))
    }
}

fn parse_training_data(
    input: impl io::Read,
    feature_size: usize,
) -> anyhow::Result<(Vec<f64>, Vec<f64>)> {
    let reader = BufReader::new(input);
    let mut targets = Vec::<f64>::new();
    let mut features = Vec::new();

    for line_result in reader.lines() {
        let line = line_result?;
        let trimed_line = line.trim();
        anyhow::ensure!(!trimed_line.is_empty(), "Empty line");

        let mut v: Vec<f64> = trimed_line
            .split(',')
            .map(|x| x.parse::<f64>())
            .collect::<std::result::Result<_, _>>()?;

        anyhow::ensure!(
            v.len() == feature_size + 1,
            "Data format error: column len = {}, expected = {}",
            v.len(),
            feature_size + 1
        );

        let label = v.swap_remove(feature_size);
        targets.push(label);
        features.extend(v);
    }

    Ok((features, targets))
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use std::path::Path;
    use std::untrusted::fs;
    use teaclave_runtime::*;
    use teaclave_test_utils::*;
    use teaclave_types::*;

    pub fn run_tests() -> bool {
        run_tests!(test_logistic_regression_training)
    }

    fn test_logistic_regression_training() {
        let func_args = FunctionArguments::new(hashmap! {
            "alg_alpha" => "0.3",
            "alg_iters" => "100",
            "feature_size" => "30"
        });

        let base = Path::new("fixtures/functions/logistic_regression_training");
        let training_data = base.join("train.txt");
        let plain_output = base.join("model.txt.out");
        let expected_output = base.join("expected_model.txt");

        let input_files = StagedFiles::new(hashmap!(
            TRAINING_DATA =>
            StagedFileInfo::new(&training_data, TeaclaveFile128Key::random()),
        ));

        let output_files = StagedFiles::new(hashmap!(
            OUT_MODEL_FILE =>
            StagedFileInfo::new(&plain_output, TeaclaveFile128Key::random())
        ));

        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));

        let function = LogitRegTraining;
        let summary = function.execute(runtime, func_args).unwrap();
        assert_eq!(summary, "Trained 100 lines of data.");

        let _result = fs::read_to_string(&plain_output).unwrap();
        let _expected = fs::read_to_string(&expected_output).unwrap();
        // assert_eq!(&result[..], &expected[..]);
    }
}
