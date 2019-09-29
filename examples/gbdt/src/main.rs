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

use lazy_static::lazy_static;
use mesatee_sdk::{Mesatee, MesateeEnclaveInfo};
use serde_derive::Serialize;
use serde_json;
use std::net::SocketAddr;
use std::{env, fs};

static FUNCTION_TRAIN_NAME: &str = "gbdt_train";
static FUNCTION_PREDICT_NAME: &str = "gbdt_predict";

static USER_ID: &str = "uid1";
static USER_TOKEN: &str = "token1";

lazy_static! {
    static ref TMS_ADDR: SocketAddr = "127.0.0.1:5554".parse().unwrap();
    static ref TDFS_ADDR: SocketAddr = "127.0.0.1:5065".parse().unwrap();
}

#[derive(Serialize)]
pub(crate) struct GBDTTrainPayload {
    gbdt_train_feature_size: usize,
    gbdt_train_max_depth: u32,
    gbdt_train_iterations: usize,
    gbdt_train_shrinkage: f64,
    gbdt_train_feature_sample_ratio: f64,
    gbdt_train_data_sample_ratio: f64,
    gbdt_train_min_leaf_size: usize,
    gbdt_train_loss: String,
    gbdt_train_debug: bool,
    gbdt_train_initial_guess_enable: bool,
    gbdt_train_training_optimization_level: u8,
}

fn print_gbdt_train_usage() {
    let msg = "
    ./gbdt 
    train 
    gbdt.train.cfg.feature_size                 [usize, default 1] 
    gbdt.train.cfg.max_depth                    [u32, default 2]
    gbdt.train.cfg.iterations                   [usize ,default 2]
    gbdt.train.cfg.shrinkage                    [f64, default 1.0]
    gbdt.train.cfg.feature_sample_ratio         [f64, default 1.0]
    gbdt.train.cfg.data_sample_ratio            [f64, default 1.0]
    gbdt.train.cfg.min_leaf_size                [usize, default 1]
    gbdt.train.cfg.loss                         [String, default SquareError]
    gbdt.train.cfg.debug                        [bool, default false]
    gbdt.train.cfg.initial_guess_enable         [bool, default false]
    gbdt.train.cfg.training_optimization_level  [u8, default 2]
    gbdt.train.data.path                        [String]
    model_file_id_saving_path                   [String]

    gbdt.train.data format:
        f32,f32,f32,f32 ...
        f32,f32,f32,f32 ...
        ...
    ";
    println!("usage: \n{}", msg);
}

fn print_gbdt_predict_usage() {
    let msg = "
    ./gbdt 
    preditct 
    gbdt.predict.test_data_path 
    gbdt.predict.model_file_id
    result_saving_path

    test data format:
        f32,f32,f32,f32 ...
        f32,f32,f32,f32 ...
        ...
    supported model:
        model saved by gbdt train
    output:
        f32
        f32
        ...
    ";
    println!("usage: \n{}", msg);
}

fn print_usage() {
    print_gbdt_train_usage();
    print_gbdt_predict_usage();
}

fn main() {
    let auditors = vec![
        (
            "../auditors/godzilla/godzilla.public.der",
            "../auditors/godzilla/godzilla.sign.sha256",
        ),
        (
            "../auditors/optimus_prime/optimus_prime.public.der",
            "../auditors/optimus_prime/optimus_prime.sign.sha256",
        ),
        (
            "../auditors/albus_dumbledore/albus_dumbledore.public.der",
            "../auditors/albus_dumbledore/albus_dumbledore.sign.sha256",
        ),
    ];

    let enclave_info_file_path = "../out/enclave_info.txt";

    let mesatee_enclave_info = MesateeEnclaveInfo::load(auditors, enclave_info_file_path).unwrap();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let action = args[1].as_str();
    match action {
        "train" => {
            if args.len() != 15 {
                print_gbdt_train_usage();
                return;
            }

            gbdt_train(
                &mesatee_enclave_info,
                args[2].parse().unwrap(),  // gbdt.train.cfg.feature_size
                args[3].parse().unwrap(),  // gbdt.train.cfg.max_depth
                args[4].parse().unwrap(),  // gbdt.train.cfg.iterations
                args[5].parse().unwrap(),  // gbdt.train.cfg.shrinkage
                args[6].parse().unwrap(),  // gbdt.train.cfg.feature_sample_ratio
                args[7].parse().unwrap(),  // gbdt.train.cfg.data_sample_ratio
                args[8].parse().unwrap(),  // gbdt.train.cfg.min_leaf_size
                &args[9],                  // gbdt.train.cfg.loss
                args[10].parse().unwrap(), // gbdt.train.cfg.debug
                args[11].parse().unwrap(), // gbdt.train.cfg.initial_guess_enable
                args[12].parse().unwrap(), // gbdt.train.cfg.training_optimization_level
                &args[13],                 // gbdt.train.data.path
                &args[14],                 // model_file_id_saving_path
            );
        }
        "predict" => {
            if args.len() != 5 {
                print_gbdt_predict_usage();
                return;
            }

            gbdt_predict(
                &mesatee_enclave_info,
                &args[2], // gbdt.predict.test_data_path
                &args[3], // gbdt.predict.model_file_id
                &args[4], // result_saving_path
            );
        }
        _ => {
            print_usage();
        }
    }
}

fn gbdt_train(
    info: &MesateeEnclaveInfo,
    gbdt_train_cfg_feature_size: usize,
    gbdt_train_cfg_max_depth: u32,
    gbdt_train_cfg_iterations: usize,
    gbdt_train_cfg_shrinkage: f64,
    gbdt_train_cfg_feature_sample_ratio: f64,
    gbdt_train_cfg_data_sample_ratio: f64,
    gbdt_train_cfg_min_leaf_size: usize,
    gbdt_train_cfg_loss: &str,
    gbdt_train_cfg_debug: bool,
    gbdt_train_cfg_initial_guess_enable: bool,
    gbdt_train_training_optimization_level: u8,
    gbdt_train_data_path: &str,
    model_file_id_saving_path: &str,
) {
    let mesatee = Mesatee::new(info, USER_ID, USER_TOKEN, *TMS_ADDR, *TDFS_ADDR).unwrap();
    let gbdt_train_file_id = mesatee.upload_file(gbdt_train_data_path).unwrap();

    let task = mesatee
        .create_task_with_files(FUNCTION_TRAIN_NAME, &[&gbdt_train_file_id])
        .unwrap();

    let input_payload = GBDTTrainPayload {
        gbdt_train_feature_size: gbdt_train_cfg_feature_size,
        gbdt_train_max_depth: gbdt_train_cfg_max_depth,
        gbdt_train_iterations: gbdt_train_cfg_iterations,
        gbdt_train_shrinkage: gbdt_train_cfg_shrinkage,
        gbdt_train_feature_sample_ratio: gbdt_train_cfg_feature_sample_ratio,
        gbdt_train_data_sample_ratio: gbdt_train_cfg_data_sample_ratio,
        gbdt_train_min_leaf_size: gbdt_train_cfg_min_leaf_size,
        gbdt_train_loss: gbdt_train_cfg_loss.to_string(),
        gbdt_train_debug: gbdt_train_cfg_debug,
        gbdt_train_initial_guess_enable: gbdt_train_cfg_initial_guess_enable,
        gbdt_train_training_optimization_level: gbdt_train_training_optimization_level,
    };

    let input_string = serde_json::to_string(&input_payload).unwrap();
    let model_file_id = task.invoke_with_payload(&input_string).unwrap();

    println!("the gbdt train model file id: \n{}", model_file_id);
    fs::write(model_file_id_saving_path, model_file_id.as_bytes()).unwrap();
}

fn gbdt_predict(
    info: &MesateeEnclaveInfo,
    gbdt_predict_test_data_path: &str,
    gbdt_predict_model_file_id: &str,
    result_saving_path: &str,
) {
    let mesatee = Mesatee::new(info, USER_ID, USER_TOKEN, *TMS_ADDR, *TDFS_ADDR).unwrap();
    // upload model_file_id and test_data_file to mesatee's tdfs
    // let model_file_id = mesatee.upload_file(model_data_path).unwrap();
    let test_file_id = mesatee.upload_file(gbdt_predict_test_data_path).unwrap();

    let file_ids: [&str; 2] = [gbdt_predict_model_file_id, test_file_id.as_str()];

    let task = mesatee
        .create_task_with_files(FUNCTION_PREDICT_NAME, &file_ids)
        .unwrap();
    let result = task.invoke().unwrap();

    println!("result:\n{}", result);
    fs::write(result_saving_path, result.as_bytes()).unwrap();
}
