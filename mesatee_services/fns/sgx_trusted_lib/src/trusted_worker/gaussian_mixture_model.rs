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
use std::fmt::Write;
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::worker::{FunctionType, Worker, WorkerContext};
use mesatee_core::{Error, ErrorKind, Result};

use rusty_machine::learning::gmm::{CovOption, GaussianMixtureModel};
use rusty_machine::learning::UnSupModel;
use rusty_machine::linalg::Matrix;

use serde_derive::Deserialize;
use serde_json;

#[derive(Deserialize)]
pub(crate) struct GmmPayload {
    k: usize,
    max_iter_num: usize,
    input_model_columns: usize,
    input_model_data: String,
    test_data: String,
}

pub struct GmmWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<GmmInput>,
}

struct GmmInput {
    k: usize,
    /// the max number of iterations for the EM algorithm.
    max_iter_num: usize,
    /// Sample model data
    input_model_data: Matrix<f64>,
    /// Data to be tested
    test_data: Matrix<f64>,
}

impl GmmWorker {
    pub fn new() -> Self {
        GmmWorker {
            worker_id: 0,
            func_name: "gaussian_mixture_model".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}

impl Worker for GmmWorker {
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

        let gmm_payload: GmmPayload = serde_json::from_str(&payload)
            .or_else(|_| Err(Error::from(ErrorKind::InvalidInputError)))?;

        let input = parse_input_to_matrix(
            &gmm_payload.input_model_data,
            gmm_payload.input_model_columns,
        )?;
        let test_data =
            parse_input_to_matrix(&gmm_payload.test_data, gmm_payload.input_model_columns)?;
        self.input = Some(GmmInput {
            k: gmm_payload.k,
            max_iter_num: gmm_payload.max_iter_num,
            input_model_data: input,
            test_data,
        });
        Ok(())
    }

    fn execute(&mut self, _context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        // Create gmm with k(=2) classes.
        let mut gmm_mod = GaussianMixtureModel::new(input.k);
        gmm_mod.set_max_iters(input.max_iter_num);
        gmm_mod.cov_option = CovOption::Diagonal;
        // Train the model
        gmm_mod
            .train(&input.input_model_data)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
        // predict a new test data
        let classes = gmm_mod
            .predict(&input.test_data)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;

        let mut output = String::new();
        for c in classes.data().iter() {
            writeln!(&mut output, "{}", c)
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
