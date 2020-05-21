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

use std::convert::TryFrom;
use std::format;
use std::io::{self, BufRead, BufReader, Write};

use teaclave_types::{FunctionArguments, FunctionRuntime};

use rusty_machine::learning::logistic_reg::LogisticRegressor;
use rusty_machine::learning::optim::grad_desc::GradientDesc;
use rusty_machine::learning::SupModel;
use rusty_machine::linalg;

const TRAINING_DATA: &str = "training_data";
const OUT_MODEL_FILE: &str = "model_file";

#[derive(Default)]
pub struct LogisticRegressionTrain;

#[derive(serde::Deserialize)]
struct LogisticRegressionTrainArguments {
    alg_alpha: f64,
    alg_iters: usize,
    feature_size: usize,
}

impl TryFrom<FunctionArguments> for LogisticRegressionTrainArguments {
    type Error = anyhow::Error;

    fn try_from(arguments: FunctionArguments) -> Result<Self, Self::Error> {
        use anyhow::Context;
        serde_json::from_str(&arguments.into_string()).context("Cannot deserialize arguments")
    }
}

impl LogisticRegressionTrain {
    pub const NAME: &'static str = "builtin-logistic-regression-train";

    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(
        &self,
        arguments: FunctionArguments,
        runtime: FunctionRuntime,
    ) -> anyhow::Result<String> {
        let args = LogisticRegressionTrainArguments::try_from(arguments)?;

        let input = runtime.open_input(TRAINING_DATA)?;
        let (flattend_features, targets) = parse_training_data(input, args.feature_size)?;
        let data_size = targets.len();
        let data_matrix = linalg::Matrix::new(data_size, args.feature_size, flattend_features);
        let targets = linalg::Vector::new(targets);

        let gd = GradientDesc::new(args.alg_alpha, args.alg_iters);
        let mut lr = LogisticRegressor::new(gd);
        lr.train(&data_matrix, &targets)?;

        let model_json = serde_json::to_string(&lr)?;
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
    use serde_json::json;
    use std::path::Path;
    use std::untrusted::fs;
    use teaclave_crypto::*;
    use teaclave_runtime::*;
    use teaclave_test_utils::*;
    use teaclave_types::*;

    pub fn run_tests() -> bool {
        run_tests!(test_logistic_regression_train)
    }

    fn test_logistic_regression_train() {
        let arguments = FunctionArguments::from_json(json!({
            "alg_alpha": 0.3,
            "alg_iters": 100,
            "feature_size": 30
        }))
        .unwrap();

        let base = Path::new("fixtures/functions/logistic_regression_training");
        let training_data = base.join("train.txt");
        let plain_output = base.join("model.txt.out");
        let expected_output = base.join("expected_model.txt");

        let input_files = StagedFiles::new(hashmap!(
            TRAINING_DATA =>
            StagedFileInfo::new(&training_data, TeaclaveFile128Key::random(), FileAuthTag::mock()),
        ));

        let output_files = StagedFiles::new(hashmap!(
            OUT_MODEL_FILE =>
            StagedFileInfo::new(&plain_output, TeaclaveFile128Key::random(), FileAuthTag::mock())
        ));

        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));

        let summary = LogisticRegressionTrain::new()
            .run(arguments, runtime)
            .unwrap();
        assert_eq!(summary, "Trained 100 lines of data.");

        let _result = fs::read_to_string(&plain_output).unwrap();
        let _expected = fs::read_to_string(&expected_output).unwrap();
        // assert_eq!(&result[..], &expected[..]);
    }
}
