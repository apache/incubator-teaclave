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

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::trait_defs::{WorkerHelper, WorkerInput};
use mesatee_core::{Error, ErrorKind, Result};

use rusty_machine::learning::lin_reg::LinRegressor;
use rusty_machine::learning::SupModel;
use rusty_machine::linalg::Matrix;
use rusty_machine::linalg::Vector;

use serde_derive::Deserialize;
use serde_json;

#[derive(Deserialize)]
pub(crate) struct LinRegPayload {
    input_mode_columns: usize,
    input_mode_data: String,
    target_mode_data: String,
    test_data: String,
}

fn data_to_vector(input: &str) -> Result<Vector<f64>> {
    let mut raw_cluster_data = Vec::new();

    for c in input.lines() {
        let value = c.parse::<f64>().unwrap();
        raw_cluster_data.push(value);
    }

    let target_data = Vector::new(raw_cluster_data);
    Ok(target_data)
}

fn parse_input_to_matrix(input: &str, input_mode_data_columns: usize) -> Result<Matrix<f64>> {
    let mut raw_cluster_data = Vec::new();

    let lines: Vec<&str> = input.split('\n').collect();
    let mut sample_num = 0;
    for line in lines.iter() {
        let trimed_line = line.trim();
        if trimed_line.is_empty() {
            continue;
        }
        let mut point: Vec<f64> = Vec::new();
        let features = trimed_line.split(',');

        for feature_str in features {
            let trimed_feature_str = feature_str.trim();
            if trimed_feature_str.is_empty() {
                continue;
            }
            let feature: f64 = trimed_feature_str
                .parse()
                .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
            point.push(feature);
        }
        if point.len() == input_mode_data_columns {
            sample_num += 1;
            raw_cluster_data.extend(point);
        }
    }

    let samples = Matrix::new(sample_num, input_mode_data_columns, raw_cluster_data);
    Ok(samples)
}

pub(crate) fn cluster(_helper: &mut WorkerHelper, input: WorkerInput) -> Result<String> {
    let payload = input
        .payload
        .ok_or_else(|| Error::from(ErrorKind::MissingValue))?;

    let lin_reg_payload: LinRegPayload = serde_json::from_str(&payload)?;
    let inputs = parse_input_to_matrix(
        &lin_reg_payload.input_mode_data,
        lin_reg_payload.input_mode_columns,
    )?;
    let targets = data_to_vector(&lin_reg_payload.target_mode_data)?;
    let test_datas = parse_input_to_matrix(
        &lin_reg_payload.test_data,
        lin_reg_payload.input_mode_columns,
    )?;

    let mut lin_mod = LinRegressor::default();
    lin_mod.train(&inputs, &targets).unwrap();
    let output = lin_mod.predict(&test_datas).unwrap();

    Ok(output[0].to_string())
}
