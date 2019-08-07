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

use crate::worker::{FunctionType, Worker, WorkerContext};
use mesatee_core::{Error, ErrorKind, Result};

use rusty_machine::learning::logistic_reg::LogisticRegressor;
use rusty_machine::learning::SupModel;
use rusty_machine::linalg::Matrix;
use rusty_machine::linalg::Vector;

use serde_derive::Deserialize;
use serde_json;

#[derive(Deserialize)]
pub(crate) struct LogisticRegPayload {
    input_model_columns: usize,
    input_model_data: String,
    target_model_data: String,
    test_data: String,
}

pub struct LogisticRegWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<LogisticRegInput>,
}

struct LogisticRegInput {
    /// Sample model data
    input_model_data: Matrix<f64>,
    /// The target(also called label or result) of the sample model data
    target_model_data: Vector<f64>,
    /// Data to be tested
    test_data: Matrix<f64>,
}

impl LogisticRegWorker {
    pub fn new() -> Self {
        LogisticRegWorker {
            worker_id: 0,
            func_name: "logistic_reg".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}

impl Worker for LogisticRegWorker {
    fn function_name(&self) -> &str {
        self.func_name.as_str()
    }
    fn function_type(&self) -> FunctionType {
        self.func_type
    }
    fn set_id(&mut self, worker_id: u32) {
        self.worker_id = worker_id;
    }
    fn id(&self) -> u32 {
        self.worker_id
    }
    fn prepare_input(
        &mut self,
        dynamic_input: Option<String>,
        _file_ids: Vec<String>,
    ) -> Result<()> {
        let payload = dynamic_input.ok_or_else(|| Error::from(ErrorKind::MissingValue))?;

        let logistic_reg_payload: LogisticRegPayload = serde_json::from_str(&payload)
            .or_else(|_| Err(Error::from(ErrorKind::InvalidInputError)))?;

        let input = parse_input_to_matrix(
            &logistic_reg_payload.input_model_data,
            logistic_reg_payload.input_model_columns,
        )?;
        let target = data_to_vector(&logistic_reg_payload.target_model_data)?;
        let test_data = parse_input_to_matrix(
            &logistic_reg_payload.test_data,
            logistic_reg_payload.input_model_columns,
        )?;
        self.input = Some(LogisticRegInput {
            input_model_data: input,
            target_model_data: target,
            test_data,
        });
        Ok(())
    }
    fn execute(&mut self, _context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;

        let mut logistic_mod = LogisticRegressor::default();
        // Train the model
        logistic_mod
            .train(&input.input_model_data, &input.target_model_data)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
        // predict a new test data
        let output = logistic_mod
            .predict(&input.test_data)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;

        Ok(output[0].to_string())
    }
}

fn data_to_vector(input: &str) -> Result<Vector<f64>> {
    let mut raw_cluster_data = Vec::new();

    for c in input.lines() {
        let value = c
            .parse::<f64>()
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
        raw_cluster_data.push(value);
    }

    let target_data = Vector::new(raw_cluster_data);
    Ok(target_data)
}

fn parse_input_to_matrix(input: &str, input_model_data_columns: usize) -> Result<Matrix<f64>> {
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
        if point.len() == input_model_data_columns {
            sample_num += 1;
            raw_cluster_data.extend(point);
        }
    }

    let samples = Matrix::new(sample_num, input_model_data_columns, raw_cluster_data);
    Ok(samples)
}
