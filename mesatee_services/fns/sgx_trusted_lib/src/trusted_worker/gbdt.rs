// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::trait_defs::{WorkerHelper, WorkerInput};
use gbdt_sgx::decision_tree::Data;
use gbdt_sgx::gradient_boost::GBDT;
use mesatee_core::{Error, ErrorKind, Result};
use serde_json;
use std::fmt::Write;
use std::panic;
use std::prelude::v1::*;

fn parse_input(input: &str) -> Result<Vec<Data>> {
    let mut samples: Vec<Data> = Vec::new();
    let lines: Vec<&str> = input.split('\n').collect();
    for line in lines.iter() {
        let trimed_line = line.trim();
        if trimed_line.is_empty() {
            continue;
        }
        let mut features: Vec<f32> = Vec::new();
        for feature_str in trimed_line.split(',') {
            let trimed_feature_str = feature_str.trim();
            if trimed_feature_str.is_empty() {
                continue;
            }
            let feature: f32 = trimed_feature_str
                .parse()
                .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
            features.push(feature);
        }
        let sample = Data::new_test_data(features, None);
        samples.push(sample);
    }
    Ok(samples)
}

pub(crate) fn predict(helper: &mut WorkerHelper, input: WorkerInput) -> Result<String> {
    let file_id = match input.input_files.get(0) {
        Some(value) => value,
        None => return Err(Error::from(ErrorKind::MissingValue)),
    };
    let sample_str = match input.payload {
        Some(value) => value,
        None => return Err(Error::from(ErrorKind::MissingValue)),
    };
    let plaintext = helper.read_file(&file_id)?;
    let serialized_model =
        String::from_utf8(plaintext).map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
    let model: GBDT = serde_json::from_str(&serialized_model)
        .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
    let samples = parse_input(&sample_str)?;
    let result = panic::catch_unwind(|| model.predict(&samples));
    let predict_set = result.map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
    let mut output = String::new();
    for predict_value in predict_set.iter() {
        writeln!(&mut output, "{:.10}", predict_value)
            .map_err(|_| Error::from(ErrorKind::OutputGenerationError))?;
    }
    Ok(output)
}
