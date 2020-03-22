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

use crate::function::TeaclaveFunction;
use crate::runtime::TeaclaveRuntime;
use teaclave_types::FunctionArguments;

use gbdt::config::Config;
use gbdt::decision_tree::Data;
use gbdt::gradient_boost::GBDT;

#[derive(Default)]
pub struct GbdtTraining;

static IN_DATA: &str = "training_data";
static OUT_MODEL: &str = "trained_model";

impl TeaclaveFunction for GbdtTraining {
    fn execute(
        &self,
        runtime: Box<dyn TeaclaveRuntime + Send + Sync>,
        arguments: FunctionArguments,
    ) -> anyhow::Result<String> {
        log::debug!("start traning...");
        let feature_size = arguments.get("feature_size")?.as_usize()?;
        let max_depth = arguments.get("max_depth")?.as_u32()?;
        let iterations = arguments.get("iterations")?.as_usize()?;
        let shrinkage = arguments.get("shrinkage")?.as_f32()?;
        let feature_sample_ratio = arguments.get("feature_sample_ratio")?.as_f64()?;
        let data_sample_ratio = arguments.get("data_sample_ratio")?.as_f64()?;
        let min_leaf_size = arguments.get("min_leaf_size")?.as_usize()?;
        let loss = arguments.get("loss")?.as_str();
        let training_optimization_level = arguments.get("training_optimization_level")?.as_u8()?;

        log::debug!("open input...");
        // read input
        let training_file = runtime.open_input(IN_DATA)?;
        let mut train_dv = parse_training_data(training_file, feature_size)?;
        let data_size = train_dv.len();

        // init gbdt config
        let mut cfg = Config::new();
        cfg.set_debug(false);
        cfg.set_feature_size(feature_size);
        cfg.set_max_depth(max_depth);
        cfg.set_iterations(iterations);
        cfg.set_shrinkage(shrinkage);
        cfg.set_loss(loss);
        cfg.set_min_leaf_size(min_leaf_size);
        cfg.set_data_sample_ratio(data_sample_ratio);
        cfg.set_feature_sample_ratio(feature_sample_ratio);
        cfg.set_training_optimization_level(training_optimization_level);

        // start training
        let mut gbdt_train_mod = GBDT::new(&cfg);
        gbdt_train_mod.fit(&mut train_dv);
        let model_json = serde_json::to_string(&gbdt_train_mod)?;

        log::debug!("create file...");
        // save the model to output
        let mut model_file = runtime.create_output(OUT_MODEL)?;
        model_file.write_all(model_json.as_bytes())?;

        let summary = format!("Trained {} lines of data.", data_size);
        Ok(summary)
    }
}

fn parse_data_line(line: &str, feature_size: usize) -> anyhow::Result<Data> {
    let trimed_line = line.trim();
    anyhow::ensure!(!trimed_line.is_empty(), "Empty line");

    let mut v: Vec<f32> = trimed_line
        .split(',')
        .map(|x| x.parse::<f32>())
        .collect::<std::result::Result<_, _>>()?;

    anyhow::ensure!(
        v.len() == feature_size + 1,
        "Data format error: column len = {}, expected = {}",
        v.len(),
        feature_size + 1
    );

    // Last column is the label
    Ok(Data {
        label: v.swap_remove(feature_size),
        feature: v,
        target: 0.0,
        weight: 1.0,
        residual: 0.0,
        initial_guess: 0.0,
    })
}

fn parse_training_data(input: impl io::Read, feature_size: usize) -> anyhow::Result<Vec<Data>> {
    let mut samples: Vec<Data> = Vec::new();
    let reader = BufReader::new(input);
    for line_result in reader.lines() {
        let line = line_result?;
        let data = parse_data_line(&line, feature_size)?;
        samples.push(data);
    }

    Ok(samples)
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use teaclave_test_utils::*;

    use std::untrusted::fs;

    use teaclave_types::hashmap;
    use teaclave_types::FunctionArguments;
    use teaclave_types::StagedFiles;
    use teaclave_types::StagedInputFile;
    use teaclave_types::StagedOutputFile;
    use teaclave_types::TeaclaveFileRootKey128;

    use crate::function::TeaclaveFunction;
    use crate::runtime::RawIoRuntime;

    pub fn run_tests() -> bool {
        run_tests!(test_gbdt_training, test_gbdt_parse_training_data,)
    }

    fn test_gbdt_training() {
        let func_arguments = FunctionArguments::from_map(&hashmap!(
            "feature_size"  => "4",
            "max_depth"     => "4",
            "iterations"    => "100",
            "shrinkage"     => "0.1",
            "feature_sample_ratio" => "1.0",
            "data_sample_ratio" => "1.0",
            "min_leaf_size" => "1",
            "loss"          => "LAD",
            "training_optimization_level" => "2"
        ));

        let plain_input = "fixtures/functions/gbdt_training/train.txt";
        let plain_output = "fixtures/functions/gbdt_training/training_model.txt.out";
        let expected_output = "fixtures/functions/gbdt_training/expected_model.txt";

        let input_files = StagedFiles::new(hashmap!(
            IN_DATA.to_string() =>
            StagedInputFile::new(plain_input, TeaclaveFileRootKey128::random())
        ));

        let output_files = StagedFiles::new(hashmap!(
            OUT_MODEL.to_string() =>
            StagedOutputFile::new(plain_output, TeaclaveFileRootKey128::random())
        ));

        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));

        let function = GbdtTraining;
        let summary = function.execute(runtime, func_arguments).unwrap();
        assert_eq!(summary, "Trained 120 lines of data.");

        let result = fs::read_to_string(&plain_output).unwrap();
        let expected = fs::read_to_string(&expected_output).unwrap();
        assert_eq!(&result[..], &expected[..]);
    }

    fn test_gbdt_parse_training_data() {
        let line = "4.8,3.0,1.4,0.3,3.0";
        let result = parse_data_line(&line, 4);
        assert!(result.is_ok());

        let result = parse_data_line(&line, 3);
        assert!(result.is_err());
    }
}
