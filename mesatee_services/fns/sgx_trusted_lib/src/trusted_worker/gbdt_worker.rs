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
use mesatee_core::{Error, ErrorKind, Result};
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use gbdt::config::Config;
use gbdt::decision_tree::Data;
use gbdt::gradient_boost::GBDT;
use serde_derive::Deserialize;
use serde_json;

use std::fmt::Write;
use std::panic;

#[derive(Deserialize)]
pub(crate) struct GBDTTrainPayload {
    gbdt_train_feature_size: usize,
    gbdt_train_max_depth: u32,
    gbdt_train_iterations: usize,
    gbdt_train_shrinkage: f32,
    gbdt_train_feature_sample_ratio: f64,
    gbdt_train_data_sample_ratio: f64,
    gbdt_train_min_leaf_size: usize,
    gbdt_train_loss: String,
    gbdt_train_training_optimization_level: u8,
}

pub struct GBDTTrainWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<GBDTTrainWorkerInput>,
}

struct GBDTTrainWorkerInput {
    gbdt_train_feature_size: usize,
    gbdt_train_max_depth: u32,
    gbdt_train_iterations: usize,
    gbdt_train_shrinkage: f32,
    gbdt_train_feature_sample_ratio: f64,
    gbdt_train_data_sample_ratio: f64,
    gbdt_train_min_leaf_size: usize,
    gbdt_train_loss: String,
    gbdt_train_training_optimization_level: u8,

    train_data_file_id: String,
}

impl GBDTTrainWorker {
    pub fn new() -> Self {
        GBDTTrainWorker {
            worker_id: 0,
            func_name: "gbdt_train".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}

impl Worker for GBDTTrainWorker {
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
        let payload = dynamic_input.ok_or_else(|| Error::from(ErrorKind::MissingValue))?;
        let gbdt_train_payload: GBDTTrainPayload = serde_json::from_str(&payload)
            .or_else(|_| Err(Error::from(ErrorKind::InvalidInputError)))?;
        let train_data_file_id = match file_ids.get(0) {
            Some(value) => value.to_string(),
            None => return Err(Error::from(ErrorKind::InvalidInputError)),
        };

        self.input = Some(GBDTTrainWorkerInput {
            gbdt_train_feature_size: gbdt_train_payload.gbdt_train_feature_size,
            gbdt_train_max_depth: gbdt_train_payload.gbdt_train_max_depth,
            gbdt_train_iterations: gbdt_train_payload.gbdt_train_iterations,
            gbdt_train_shrinkage: gbdt_train_payload.gbdt_train_shrinkage,
            gbdt_train_feature_sample_ratio: gbdt_train_payload.gbdt_train_feature_sample_ratio,
            gbdt_train_data_sample_ratio: gbdt_train_payload.gbdt_train_data_sample_ratio,
            gbdt_train_min_leaf_size: gbdt_train_payload.gbdt_train_min_leaf_size,
            gbdt_train_loss: gbdt_train_payload.gbdt_train_loss,
            gbdt_train_training_optimization_level: gbdt_train_payload
                .gbdt_train_training_optimization_level,
            train_data_file_id,
        });
        Ok(())
    }

    fn execute(&mut self, context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;

        let train_data_bytes = context.read_file(&input.train_data_file_id)?;

        let train_data_str = String::from_utf8(train_data_bytes)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;

        let mut train_dv = parse_train_data(&train_data_str)?;
        // init gbdt config
        let mut cfg = Config::new();
        cfg.set_feature_size(input.gbdt_train_feature_size);
        cfg.set_max_depth(input.gbdt_train_max_depth);
        cfg.set_iterations(input.gbdt_train_iterations);
        cfg.set_shrinkage(input.gbdt_train_shrinkage);
        cfg.set_loss(&input.gbdt_train_loss);
        cfg.set_debug(false);
        cfg.set_min_leaf_size(input.gbdt_train_min_leaf_size);
        cfg.set_data_sample_ratio(input.gbdt_train_data_sample_ratio);
        cfg.set_feature_sample_ratio(input.gbdt_train_feature_sample_ratio);
        cfg.set_training_optimization_level(input.gbdt_train_training_optimization_level);

        let mut gbdt_train_mod = GBDT::new(&cfg);
        gbdt_train_mod.fit(&mut train_dv);
        let model_json = serde_json::to_string(&gbdt_train_mod).unwrap();

        // save the model
        context.save_file_for_task_creator(model_json.as_bytes())
    }
}

pub struct GBDTPredictWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<GBDTPredictWorkerInput>,
}
struct GBDTPredictWorkerInput {
    model_file_id: String,
    test_file_id: String,
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
        _dynamic_input: Option<String>,
        file_ids: Vec<String>,
    ) -> Result<()> {
        let model_file_id = match file_ids.get(0) {
            Some(value) => value.to_string(),
            None => return Err(Error::from(ErrorKind::InvalidInputError)),
        };

        let test_file_id = match file_ids.get(1) {
            Some(value) => value.to_string(),
            None => return Err(Error::from(ErrorKind::InvalidInputError)),
        };
        self.input = Some(GBDTPredictWorkerInput {
            model_file_id,
            test_file_id,
        });
        Ok(())
    }

    fn execute(&mut self, context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;

        let model_bytes: Vec<u8> = context.read_file(&input.model_file_id)?;
        let test_data_bytes: Vec<u8> = context.read_file(&input.test_file_id)?;

        let model_json_str = String::from_utf8(model_bytes)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
        let test_data_str = String::from_utf8(test_data_bytes)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
        let test_data_dv = parse_test_data(&test_data_str)?;

        let model: GBDT = serde_json::from_str(&model_json_str)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
        let result = panic::catch_unwind(|| model.predict(&test_data_dv));
        let predict_set = result.map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
        let mut output = String::new();
        for predict_value in predict_set.iter() {
            writeln!(&mut output, "{:.10}", predict_value)
                .map_err(|_| Error::from(ErrorKind::OutputGenerationError))?;
        }
        Ok(output)
    }
}

fn parse_test_data(input: &str) -> Result<Vec<Data>> {
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

fn parse_train_data(input: &str) -> Result<Vec<Data>> {
    let mut v: Vec<f32>;
    let mut samples: Vec<Data> = Vec::new();
    let mut sample_label_index = 0;
    let lines: Vec<&str> = input.split('\n').collect();

    // get the label_index
    if !lines.is_empty() && !lines[0].trim().is_empty() {
        let first_line: Vec<&str> = lines[0].trim().split(',').collect();
        if first_line.len() > 2 {
            sample_label_index = first_line.len() - 1;
        }
    }

    for line in lines.iter() {
        let trimed_line = line.trim();
        if trimed_line.is_empty() {
            continue;
        }

        v = trimed_line
            .split(',')
            .map(|x| x.parse::<f32>().unwrap_or(0.0))
            .collect();
        samples.push(Data {
            label: v.swap_remove(sample_label_index),
            feature: v,
            target: 0.0,
            weight: 1.0,
            residual: 0.0,
            initial_guess: 0.0,
        });
    }
    Ok(samples)
}
