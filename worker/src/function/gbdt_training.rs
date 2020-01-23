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

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use std::format;
use std::io::{self, BufRead, BufReader, Write};

use anyhow;
use serde_json;

use super::TeaclaveFunction;
use crate::runtime::TeaclaveRuntime;
use teaclave_types::TeaclaveFunctionArguments;

use gbdt::config::Config;
use gbdt::decision_tree::Data;
use gbdt::gradient_boost::GBDT;

#[derive(Default)]
pub struct GbdtTraining;

impl TeaclaveFunction for GbdtTraining {
    fn execute(
        &self,
        runtime: Box<dyn TeaclaveRuntime + Send + Sync>,
        args: TeaclaveFunctionArguments,
    ) -> anyhow::Result<String> {
        let feature_size: usize = args.try_get("feature_size")?;
        let max_depth: u32 = args.try_get("max_depth")?;
        let iterations: usize = args.try_get("iterations")?;
        let shrinkage: f32 = args.try_get("shrinkage")?;
        let feature_sample_ratio: f64 = args.try_get("feature_sample_ratio")?;
        let data_sample_ratio: f64 = args.try_get("data_sample_ratio")?;
        let min_leaf_size: usize = args.try_get("min_leaf_size")?;
        let loss: String = args.try_get("loss")?;
        let training_optimization_level: u8 = args.try_get("training_optimization_level")?;

        // read input
        let training_file = runtime.open_input("training_data")?;
        let mut train_dv = parse_training_data(training_file)?;
        let data_size = train_dv.len();

        // init gbdt config
        let mut cfg = Config::new();
        cfg.set_debug(false);
        cfg.set_feature_size(feature_size);
        cfg.set_max_depth(max_depth);
        cfg.set_iterations(iterations);
        cfg.set_shrinkage(shrinkage);
        cfg.set_loss(&loss);
        cfg.set_min_leaf_size(min_leaf_size);
        cfg.set_data_sample_ratio(data_sample_ratio);
        cfg.set_feature_sample_ratio(feature_sample_ratio);
        cfg.set_training_optimization_level(training_optimization_level);

        // start training
        let mut gbdt_train_mod = GBDT::new(&cfg);
        gbdt_train_mod.fit(&mut train_dv);
        let model_json = serde_json::to_string(&gbdt_train_mod)?;

        // save the model to output
        let mut model_file = runtime.create_output("trained_model")?;
        model_file.write_all(model_json.as_bytes())?;

        let summary = format!("Trained {} lines of data.", data_size);
        Ok(summary)
    }
}

fn parse_training_data(input: impl io::Read) -> anyhow::Result<Vec<Data>> {
    let mut samples: Vec<Data> = Vec::new();

    let reader = BufReader::new(input);
    for line_result in reader.lines() {
        let line = line_result?;
        let trimed_line = line.trim();
        if trimed_line.is_empty() {
            continue;
        }

        let mut v: Vec<f32> = trimed_line
            .split(',')
            .map(|x| x.parse::<f32>().unwrap_or(0.0))
            .collect();

        if v.len() <= 2 {
            continue;
        }

        let sample_label_index = v.len() - 1;
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
