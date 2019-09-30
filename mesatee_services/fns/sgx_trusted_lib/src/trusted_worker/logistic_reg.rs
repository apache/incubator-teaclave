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
use std::fmt::Write;
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use rusty_machine::learning::logistic_reg::LogisticRegressor;
use rusty_machine::learning::optim::grad_desc::GradientDesc;
use rusty_machine::learning::SupModel;
use rusty_machine::linalg::Matrix;
use rusty_machine::linalg::Vector;
use serde_derive::Deserialize;
use serde_json;

pub struct LogisticRegPredictWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<LogisticRegPredictWorkerInput>,
}

struct LogisticRegPredictWorkerInput {
    /// Sample model data
    model_file_id: String,
    /// The target(also called label or result) of the sample model data
    test_file_id: String,
}

impl LogisticRegPredictWorker {
    pub fn new() -> Self {
        LogisticRegPredictWorker {
            worker_id: 0,
            func_name: "logisticreg_predict".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}

impl Worker for LogisticRegPredictWorker {
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

        self.input = Some(LogisticRegPredictWorkerInput {
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

        let test_data_matrix = parse_input_to_matrix(&test_data_str.to_string())?;
        //info!("model json:\n{:}",model_json);
        //let mut predict = LogisticRegressor::default();
        let mut alg_alpha: f64 = 0.3;
        let mut alg_iters: u64 = 100;
        let mut param: &Vec<serde_json::Value> = &Vec::new();
        let mut check_alg_alpha = false;
        let mut check_alg_iters = false;
        let mut check_param = false;
        let model_param: serde_json::Value = serde_json::from_str(&model_json_str)?;
        if !model_param.is_null() && model_param.is_object() {
            if !model_param["alg"].is_null() && model_param["alg"].is_object() {
                if !model_param["alg"]["alpha"].is_null() && model_param["alg"]["alpha"].is_f64() {
                    // set alg_alpha
                    alg_alpha = model_param["alg"]["alpha"].as_f64().unwrap();
                    check_alg_alpha = true;
                }

                if !model_param["alg"]["iters"].is_null() && model_param["alg"]["iters"].is_u64() {
                    // set alg_iters
                    alg_iters = model_param["alg"]["iters"].as_u64().unwrap();
                    check_alg_iters = true;
                }
            }

            if !model_param["base"].is_null()
                && model_param["base"].is_object()
                && !model_param["base"]["parameters"].is_null()
                && model_param["base"]["parameters"].is_object()
                && !model_param["base"]["parameters"]["data"].is_null()
                && model_param["base"]["parameters"]["data"].is_array()
            {
                // set param
                param = model_param["base"]["parameters"]["data"]
                    .as_array()
                    .unwrap();
                check_param = true;
            }
        }

        let mut param_data = Vec::new();
        if check_alg_alpha && check_alg_iters && check_param {
            for i in param {
                param_data.push(i.as_f64().unwrap());
            }

            let base_param = Vector::new(param_data);
            let gd = GradientDesc::new(alg_alpha, alg_iters as usize);
            let mut logistic_mod = LogisticRegressor::new(gd);
            logistic_mod.set_parameters(base_param);
            let classes = logistic_mod
                .predict(&test_data_matrix)
                .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
            let mut output = String::new();
            for c in classes.data().iter() {
                writeln!(&mut output, "{}", c)
                    .map_err(|_| Error::from(ErrorKind::OutputGenerationError))?;
            }
            Ok(output)
        } else {
            Ok("something error in execute".to_string())
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct LogisticRegTrainPayload {
    train_alg_alpha: f64,
    train_alg_iters: usize,
}

pub struct LogisticRegTrainWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<LogisticRegTrainWorkerInput>,
}

struct LogisticRegTrainWorkerInput {
    /// alg.alpha
    train_alg_alpha: f64,
    /// alg.iters
    train_alg_iters: usize,
    /// Sample model data
    train_data_file_id: String,
    /// The target(also called label or result) of the sample model data
    target_data_file_id: String,
}

impl LogisticRegTrainWorker {
    pub fn new() -> Self {
        LogisticRegTrainWorker {
            worker_id: 0,
            func_name: "logisticreg_train".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}

impl Worker for LogisticRegTrainWorker {
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

        let logisticreg_train_payload: LogisticRegTrainPayload = serde_json::from_str(&payload)
            .or_else(|_| Err(Error::from(ErrorKind::InvalidInputError)))?;

        let train_data_file_id = match file_ids.get(0) {
            Some(value) => value.to_string(),
            None => return Err(Error::from(ErrorKind::InvalidInputError)),
        };

        let target_data_file_id = match file_ids.get(1) {
            Some(value) => value.to_string(),
            None => return Err(Error::from(ErrorKind::InvalidInputError)),
        };

        self.input = Some(LogisticRegTrainWorkerInput {
            train_alg_alpha: logisticreg_train_payload.train_alg_alpha,
            train_alg_iters: logisticreg_train_payload.train_alg_iters,
            train_data_file_id,
            target_data_file_id,
        });
        Ok(())
    }

    fn execute(&mut self, context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;

        let train_data_bytes = context.read_file(&input.train_data_file_id)?;
        let target_data_bytes = context.read_file(&input.target_data_file_id)?;

        let train_data_str = String::from_utf8(train_data_bytes)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
        let target_data_str = String::from_utf8(target_data_bytes)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;

        let train_data_matrix = parse_input_to_matrix(&train_data_str.to_string())?;
        let target_data_vector = parse_input_to_vector(&target_data_str.to_string())?;

        let gd = GradientDesc::new(input.train_alg_alpha, input.train_alg_iters);
        let mut logisticreg_train_mod = LogisticRegressor::new(gd);
        //let mut logisticreg_train_mod = LogisticRegressor::default();
        // train the model
        logisticreg_train_mod
            .train(&train_data_matrix, &target_data_vector)
            .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;

        // serialized the model
        let model_json = serde_json::to_string(&logisticreg_train_mod).unwrap();

        // save the model
        context.save_file_for_all_participants(model_json.as_bytes())
    }
}

fn parse_input_to_vector(input: &str) -> Result<Vector<f64>> {
    let mut raw_cluster_data = Vec::new();
    let lines: Vec<&str> = input.split('\n').collect();

    if !lines.is_empty() && !lines[0].trim().is_empty() {
        for c in input.lines() {
            let value = c
                .parse::<f64>()
                .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
            raw_cluster_data.push(value);
        }
    }

    let target_data = Vector::new(raw_cluster_data);
    Ok(target_data)
}

fn parse_input_to_matrix(input: &str) -> Result<Matrix<f64>> {
    let mut raw_cluster_data = Vec::new();

    let lines: Vec<&str> = input.split('\n').collect();
    let mut sample_num = 0;
    let columns_num = if !lines.is_empty() && !lines[0].trim().is_empty() {
        let first_line: Vec<&str> = lines[0].trim().split(',').collect();
        first_line.len()
    } else {
        0
    };

    if columns_num > 0 {
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
            if point.len() == columns_num {
                // get sample num
                sample_num += 1;
                raw_cluster_data.extend(point);
            }
        }
    }

    let samples = Matrix::new(sample_num, columns_num, raw_cluster_data);
    Ok(samples)
}
