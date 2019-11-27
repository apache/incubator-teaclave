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
use std::fmt::Write;
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::worker::{FunctionType, Worker, WorkerContext};
use mesatee_core::{Error, ErrorKind, Result};

use rusty_machine::learning::dbscan::DBSCAN;
use rusty_machine::learning::UnSupModel;
use rusty_machine::linalg::Matrix;

use serde_derive::Deserialize;
use serde_json;

#[derive(Deserialize)]
pub(crate) struct DBSCANPayload {
    eps: f64,
    min_points: usize,
    input_model_columns: usize,
    input_model_data: String,
}

pub struct DBSCANWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<DBSCANInput>,
}

struct DBSCANInput {
    // Desired region ball radius
    eps: f64,
    /// Minimum number of points to be in a region
    min_points: usize,
    /// Sample model data
    input_model_data: Matrix<f64>,
}

impl DBSCANWorker {
    pub fn new() -> Self {
        DBSCANWorker {
            worker_id: 0,
            func_name: "dbscan".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}

impl Worker for DBSCANWorker {
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

        let dbscan_payload: DBSCANPayload = serde_json::from_str(&payload)
            .or_else(|_| Err(Error::from(ErrorKind::InvalidInputError)))?;

        let input = parse_input_to_matrix(
            &dbscan_payload.input_model_data,
            dbscan_payload.input_model_columns,
        )?;

        self.input = Some(DBSCANInput {
            eps: dbscan_payload.eps,
            min_points: dbscan_payload.min_points,
            input_model_data: input,
        });
        Ok(())
    }

    fn execute(&mut self, _context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;

        let mut model = DBSCAN::new(input.eps, input.min_points);
        model
            .train(&input.input_model_data)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;

        let clustering = match model.clusters() {
            Some(v) => v,
            None => return Err(Error::from(ErrorKind::OutputGenerationError)),
        };

        let mut output = String::new();
        for c in clustering {
            let value = match c {
                Some(v) => v,
                None => return Err(Error::from(ErrorKind::OutputGenerationError)),
            };
            writeln!(&mut output, "{}", value)
                .map_err(|_| Error::from(ErrorKind::OutputGenerationError))?;
        }

        Ok(output)
    }
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
