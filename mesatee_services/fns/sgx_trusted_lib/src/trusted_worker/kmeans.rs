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

use rusty_machine::learning::k_means::KMeansClassifier;
use rusty_machine::learning::UnSupModel;
use rusty_machine::linalg::Matrix;
use serde_derive::Deserialize;
use serde_json;

#[derive(Deserialize)]
pub(crate) struct KmeansPayload {
    k: usize,
    feature_num: usize,
    data: String,
}

pub struct KmeansWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<KmeansWorkerInput>,
}

struct KmeansWorkerInput {
    k: usize,
    samples: Matrix<f64>,
}

impl KmeansWorker {
    pub fn new() -> Self {
        KmeansWorker {
            worker_id: 0,
            func_name: "kmeans_cluster".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}

impl Worker for KmeansWorker {
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

        let kmeans_payload: KmeansPayload = serde_json::from_str(&payload)
            .or_else(|_| Err(Error::from(ErrorKind::InvalidInputError)))?;

        let samples = parse_input(&kmeans_payload.data, kmeans_payload.feature_num)?;
        self.input = Some(KmeansWorkerInput {
            k: kmeans_payload.k,
            samples,
        });
        Ok(())
    }

    fn execute(&mut self, _context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        let mut model = KMeansClassifier::new(input.k);
        model
            .train(&input.samples)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;

        let mut output = String::new();
        let classes = model
            .predict(&input.samples)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
        writeln!(&mut output, "labels:")
            .map_err(|_| Error::from(ErrorKind::OutputGenerationError))?;
        for c in classes.data().iter() {
            writeln!(&mut output, "{}", c)
                .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
        }

        Ok(output)
    }
}

fn parse_input(input: &str, feature_num: usize) -> Result<Matrix<f64>> {
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
        if point.len() == feature_num {
            sample_num += 1;
            raw_cluster_data.extend(point);
        }
    }

    let samples = Matrix::new(sample_num, feature_num, raw_cluster_data);
    Ok(samples)
}
