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
use crate::worker::{FunctionType, Worker, WorkerContext};
use gbdt_sgx::decision_tree::Data;
use gbdt_sgx::gradient_boost::GBDT;
use mesatee_core::{Error, ErrorKind, Result};
use serde_json;
use std::fmt::Write;
use std::panic;
use std::prelude::v1::*;

pub struct GBDTPredictWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<GBDTPredictWorkerInput>,
}
struct GBDTPredictWorkerInput {
    samples: Vec<Data>,
    model_file_id: String,
}
impl GBDTPredictWorker {
    pub fn new() -> Self {
        GBDTPredictWorker {
            worker_id: 0,
            func_name: "gbdt_predict".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}

impl Worker for GBDTPredictWorker {
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
        file_ids: Vec<String>,
    ) -> Result<()> {
        let model_file_id = match file_ids.get(0) {
            Some(value) => value.to_string(),
            None => return Err(Error::from(ErrorKind::InvalidInputError)),
        };
        let sample_str = match dynamic_input {
            Some(value) => value,
            None => return Err(Error::from(ErrorKind::InvalidInputError)),
        };
        let samples = parse_input(&sample_str)?;
        self.input = Some(GBDTPredictWorkerInput {
            samples,
            model_file_id,
        });
        Ok(())
    }

    fn execute(&mut self, context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        let plaintext = context.read_file(&input.model_file_id)?;
        let serialized_model =
            String::from_utf8(plaintext).map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
        let model: GBDT = serde_json::from_str(&serialized_model)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
        let result = panic::catch_unwind(|| model.predict(&input.samples));
        let predict_set = result.map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
        let mut output = String::new();
        for predict_value in predict_set.iter() {
            writeln!(&mut output, "{:.10}", predict_value)
                .map_err(|_| Error::from(ErrorKind::OutputGenerationError))?;
        }
        Ok(output)
    }
}

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
