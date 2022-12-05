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

use std::convert::TryFrom;
use std::format;
use std::io::{self, BufRead, BufReader, Write};

use teaclave_types::{FunctionArguments, FunctionRuntime};

use rusty_machine::learning::pca::PCA;
use rusty_machine::learning::UnSupModel;
use rusty_machine::linalg;
use rusty_machine::linalg::BaseMatrix;
const IN_DATA: &str = "input_data";
const OUT_RESULT: &str = "output_data";

#[derive(Default)]
pub struct PrincipalComponentsAnalysis;

#[derive(serde::Deserialize)]
struct PrincipalComponentsAnalysisArguments {
    n: usize,
    center: bool,
    feature_size: usize,
}

impl TryFrom<FunctionArguments> for PrincipalComponentsAnalysisArguments {
    type Error = anyhow::Error;

    fn try_from(arguments: FunctionArguments) -> Result<Self, Self::Error> {
        use anyhow::Context;
        serde_json::from_str(&arguments.into_string()).context("Cannot deserialize arguments")
    }
}

impl PrincipalComponentsAnalysis {
    pub const NAME: &'static str = "builtin_principal_components_analysis";

    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(
        &self,
        arguments: FunctionArguments,
        runtime: FunctionRuntime,
    ) -> anyhow::Result<String> {
        let args = PrincipalComponentsAnalysisArguments::try_from(arguments)?;
        let input = runtime.open_input(IN_DATA)?;
        let (flattend_features, targets) = parse_input_data(input, args.feature_size)?;

        let data_size = targets.len();
        let input_features = linalg::Matrix::new(data_size, args.feature_size, flattend_features);

        let mut model = PCA::new(args.n, args.center);
        model.train(&input_features)?;

        let predict_result = model.predict(&input_features)?;

        let mut output = runtime.create_output(OUT_RESULT)?;
        for i in 0..predict_result.rows() {
            for j in 0..predict_result.cols() {
                if j == predict_result.cols() - 1 {
                    write!(&mut output, "{:?}", predict_result[[i, j]])?;
                } else {
                    write!(&mut output, "{:?},", predict_result[[i, j]])?;
                }
            }
            writeln!(&mut output)?;
        }

        Ok(format!(
            "transform {} rows * {} cols lines of data.",
            predict_result.rows(),
            predict_result.cols()
        ))
    }
}

fn parse_input_data(
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
        run_tests!(test_pca_predict)
    }

    fn test_pca_predict() {
        let args = FunctionArguments::from_json(json!({
            "n": 2,
            "feature_size": 4,
            "center":true
        }))
        .unwrap();

        let base = Path::new("fixtures/functions/princopal_components_analysis");

        let input_data_file = base.join("input.txt");
        let output_data_file = base.join("result.txt");
        let expected_output = base.join("expected_result.txt");

        let input_files = StagedFiles::new(hashmap!(
            IN_DATA =>
            StagedFileInfo::new(&input_data_file, TeaclaveFile128Key::random(), FileAuthTag::mock()),
        ));

        let output_files = StagedFiles::new(hashmap!(
            OUT_RESULT =>
            StagedFileInfo::new(&output_data_file, TeaclaveFile128Key::random(), FileAuthTag::mock()),
        ));

        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));
        let summary = PrincipalComponentsAnalysis::new()
            .run(args, runtime)
            .unwrap();
        assert_eq!(summary, "transform 90 rows * 2 cols lines of data.");

        let result = fs::read_to_string(&output_data_file).unwrap();
        let expected = fs::read_to_string(&expected_output).unwrap();
        assert_eq!(&result[..], &expected[..]);
    }
}
