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

use gbdt::decision_tree::Data;
use gbdt::gradient_boost::GBDT;

const IN_MODEL: &str = "model_file";
const IN_DATA: &str = "data_file";
const OUT_RESULT: &str = "result_file";

#[derive(Default)]
pub struct GbdtPrediction;

impl TeaclaveFunction for GbdtPrediction {
    fn execute(
        &self,
        runtime: FunctionRuntime,
        _arguments: FunctionArguments,
    ) -> anyhow::Result<String> {
        let mut json_model = String::new();
        let mut f = runtime.open_input(IN_MODEL)?;
        f.read_to_string(&mut json_model)?;

        let model: GBDT = serde_json::from_str(&json_model)?;

        let in_data = runtime.open_input(IN_DATA)?;
        let test_data = parse_test_data(in_data)?;

        let predict_set = model.predict(&test_data);

        let mut of_result = runtime.create_output(OUT_RESULT)?;
        for predict_value in predict_set.iter() {
            writeln!(&mut of_result, "{:.10}", predict_value)?
        }

        let summary = format!("Predict result has {} lines of data.", predict_set.len());
        Ok(summary)
    }
}

fn parse_data_line(line: &str) -> anyhow::Result<Data> {
    let trimed_line = line.trim();
    anyhow::ensure!(!trimed_line.is_empty(), "Empty line");

    let mut features: Vec<f32> = Vec::new();
    for feature_str in trimed_line.split(',') {
        let trimed_feature_str = feature_str.trim();
        anyhow::ensure!(!trimed_feature_str.is_empty(), "Empty feature");

        let feature: f32 = trimed_feature_str.parse()?;
        features.push(feature);
    }
    Ok(Data::new_test_data(features, None))
}

fn parse_test_data(input: impl io::Read) -> anyhow::Result<Vec<Data>> {
    let mut samples: Vec<Data> = Vec::new();

    let reader = BufReader::new(input);
    for line_result in reader.lines() {
        let line = line_result?;
        let data = parse_data_line(&line)?;
        samples.push(data);
    }

    Ok(samples)
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use std::untrusted::fs;
    use teaclave_crypto::*;
    use teaclave_runtime::*;
    use teaclave_test_utils::*;
    use teaclave_types::*;

    pub fn run_tests() -> bool {
        run_tests!(test_gbdt_prediction)
    }

    fn test_gbdt_prediction() {
        let func_args = FunctionArguments::default();

        let plain_model = "fixtures/functions/gbdt_prediction/model.txt";
        let plain_data = "fixtures/functions/gbdt_prediction/test_data.txt";
        let plain_output = "fixtures/functions/gbdt_prediction/result.txt.out";
        let expected_output = "fixtures/functions/gbdt_prediction/expected_result.txt";

        let input_files = StagedFiles::new(hashmap!(
            IN_MODEL =>
            StagedFileInfo::new(plain_model, TeaclaveFile128Key::random(), FileAuthTag::mock()),
            IN_DATA =>
            StagedFileInfo::new(plain_data, TeaclaveFile128Key::random(), FileAuthTag::mock())
        ));

        let output_files = StagedFiles::new(hashmap!(
            OUT_RESULT =>
            StagedFileInfo::new(plain_output, TeaclaveFile128Key::random(), FileAuthTag::mock())
        ));

        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));

        let function = GbdtPrediction;
        let summary = function.execute(runtime, func_args).unwrap();
        assert_eq!(summary, "Predict result has 30 lines of data.");

        let result = fs::read_to_string(&plain_output).unwrap();
        let expected = fs::read_to_string(&expected_output).unwrap();
        assert_eq!(&result[..], &expected[..]);
    }
}
