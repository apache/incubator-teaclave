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

use std::format;
use std::io::{self, BufRead, BufReader, Write};

use teaclave_types::{FunctionArguments, FunctionRuntime, TeaclaveFunction};

use rusty_machine::learning::logistic_reg::LogisticRegressor;
use rusty_machine::learning::optim::grad_desc::GradientDesc;
use rusty_machine::learning::SupModel;
use rusty_machine::linalg;

const MODEL_FILE: &str = "model_file";
const INPUT_DATA: &str = "data_file";
const RESULT: &str = "result_file";

#[derive(Default)]
pub struct LogitRegPrediction;

impl TeaclaveFunction for LogitRegPrediction {
    fn execute(
        &self,
        runtime: FunctionRuntime,
        _arguments: FunctionArguments,
    ) -> anyhow::Result<String> {
        let mut model_json = String::new();
        let mut f = runtime.open_input(MODEL_FILE)?;
        f.read_to_string(&mut model_json)?;

        let lr: LogisticRegressor<GradientDesc> = serde_json::from_str(&model_json)?;
        let feature_size = lr
            .parameters()
            .ok_or_else(|| anyhow::anyhow!("Model parameter is None"))?
            .size()
            - 1;

        let input = runtime.open_input(INPUT_DATA)?;
        let data_matrix = parse_input_data(input, feature_size)?;

        let result = lr.predict(&data_matrix)?;

        let mut output = runtime.create_output(RESULT)?;
        let result_cnt = result.data().len();
        for c in result.data().iter() {
            writeln!(&mut output, "{:.4}", c)?;
        }
        Ok(format!("Predicted {} lines of data.", result_cnt))
    }
}

fn parse_input_data(
    input: impl io::Read,
    feature_size: usize,
) -> anyhow::Result<linalg::Matrix<f64>> {
    let mut flattened_data = Vec::new();
    let mut count = 0;

    let reader = BufReader::new(input);
    for line_result in reader.lines() {
        let line = line_result?;
        let trimed_line = line.trim();
        anyhow::ensure!(!trimed_line.is_empty(), "Empty line");

        let v: Vec<f64> = trimed_line
            .split(',')
            .map(|x| x.parse::<f64>())
            .collect::<std::result::Result<_, _>>()?;

        anyhow::ensure!(
            v.len() == feature_size,
            "Data format error: column len = {}, expected = {}",
            v.len(),
            feature_size
        );

        flattened_data.extend(v);
        count += 1;
    }

    Ok(linalg::Matrix::new(count, feature_size, flattened_data))
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use std::path::Path;
    use std::untrusted::fs;
    use teaclave_crypto::*;
    use teaclave_runtime::*;
    use teaclave_test_utils::*;
    use teaclave_types::*;

    pub fn run_tests() -> bool {
        run_tests!(test_logistic_regression_prediction)
    }

    fn test_logistic_regression_prediction() {
        let func_args = FunctionArguments::default();

        let base = Path::new("fixtures/functions/logistic_regression_prediction");
        let model = base.join("model.txt");
        let plain_input = base.join("predict_input.txt");
        let plain_output = base.join("predict_result.txt.out");
        let expected_output = base.join("expected_result.txt");

        let input_files = StagedFiles::new(hashmap!(
            MODEL_FILE =>
            StagedFileInfo::new(&model, TeaclaveFile128Key::random()),
            INPUT_DATA =>
            StagedFileInfo::new(&plain_input, TeaclaveFile128Key::random()),
        ));

        let output_files = StagedFiles::new(hashmap!(
            RESULT =>
            StagedFileInfo::new(&plain_output, TeaclaveFile128Key::random())
        ));

        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));

        let function = LogitRegPrediction;
        let summary = function.execute(runtime, func_args).unwrap();
        assert_eq!(summary, "Predicted 5 lines of data.");

        let result = fs::read_to_string(&plain_output).unwrap();
        let expected = fs::read_to_string(&expected_output).unwrap();
        assert_eq!(&result[..], &expected[..]);
    }
}
