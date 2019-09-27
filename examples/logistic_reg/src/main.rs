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
//use std::fs;

static FUNCTION_TRAIN_NAME: &str = "logisticreg_train";
static FUNCTION_PREDICT_NAME: &str = "logisticreg_predict";

static USER_ID: &str = "uid1";
static USER_TOKEN: &str = "token1";

lazy_static! {
    static ref TMS_ADDR: SocketAddr = "127.0.0.1:5554".parse().unwrap();
    static ref TDFS_ADDR: SocketAddr = "127.0.0.1:5065".parse().unwrap();
}

#[derive(Serialize)]
pub(crate) struct LogisticRegTrainPayload {
    train_alg_alpha: f64,
    train_alg_iters: usize,
}

fn print_logisticreg_train_usage() {
    let msg = "
    ./logisticreg train logisticreg.train.alg.alpha logisticreg.train.alg.iters logisticreg.train.data.path logisticreg.target.data.path model_file_id_saving_path
    logisticreg.train.alg_alpha : f64, such as 0.3
    logisticreg.train.alg_iters : usize, such as 100 
    logisticreg.train.data format:
        f32,f32,f32,f32 ...
        f32,f32,f32,f32 ...
        ...
    logisticreg.target.data format:
        1.
        0.
        1.
        0.
        0.
        1.
        ....
    ";
    println!("logisticreg_train usage: \n{}", msg);
}

fn print_logisticreg_predict_usage() {
    let msg = "
    ./logisticreg predict logisticreg.predict.model.file.id logisticreg.predict.test.data.path predict_result_saving_path
    1.use logisticreg train to get the logisticreg.predict.mode.file.id file,then 
    2.logisticreg.predict.test.data format:
        f32,f32,f32,f32 ...
        f32,f32,f32,f32 ...
        ...
    ";
    println!("logisticreg_predict usage: \n{}", msg);
}

fn print_usage() {
    print_logisticreg_train_usage();
    print_logisticreg_predict_usage();
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
    let args_string: Vec<String> = env::args().collect();
    let args: Vec<&str> = args_string.iter().map(|s| s.as_str()).collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let action = args[1];
    match action {
        "train" => {
            if args.len() != 7 {
                print_logisticreg_train_usage();
                return;
            }

            logisticreg_train(
                &mesatee_enclave_info,
                args[2].parse().unwrap(), // logisticreg.train.alg.alpha
                args[3].parse().unwrap(), // logisticreg.train.alg.iters
                args[4],                  // logisticreg.train.data.path
                args[5],                  // logisticreg.target.data.path
                args[6],                  // model_file_id_saving_path
            );
        }
        "predict" => {
            if args.len() != 5 {
                print_logisticreg_predict_usage();
                return;
            }

            logisticreg_predict(
                &mesatee_enclave_info,
                args[2], // logisticreg.predict.model.file.id
                args[3], // logisticreg.predict.test.data.path
                args[4], // predict_result_saving_path
            );
        }
        _ => {
            print_usage();
        }
    }
}

fn logisticreg_train(
    info: &MesateeEnclaveInfo,
    alg_alpha: f64,
    alg_iters: usize,
    train_data_path: &str,
    target_data_path: &str,
    model_file_id_saving_path: &str,
) {
    let mesatee = Mesatee::new(info, USER_ID, USER_TOKEN, *TMS_ADDR, *TDFS_ADDR).unwrap();
    // upload train_data and target_data to mesatee's tdfs
    let train_file_id = mesatee.upload_file(train_data_path).unwrap();
    let target_file_id = mesatee.upload_file(target_data_path).unwrap();

    let file_ids: [&str; 2] = [train_file_id.as_str(), target_file_id.as_str()];

    let task = mesatee
        .create_task_with_files(FUNCTION_TRAIN_NAME, &file_ids)
        .unwrap();

    let input_payload = LogisticRegTrainPayload {
        train_alg_alpha: alg_alpha,
        train_alg_iters: alg_iters,
    };

    let input_string = serde_json::to_string(&input_payload).unwrap();
    let model_file_id = task.invoke_with_payload(&input_string).unwrap();

    println!("the model file id: \n{}", model_file_id);
    fs::write(model_file_id_saving_path, model_file_id.as_bytes()).unwrap();
}

fn logisticreg_predict(
    info: &MesateeEnclaveInfo,
    model_file_id: &str,
    test_data_path: &str,
    result_saving_path: &str,
) {
    let mesatee = Mesatee::new(info, USER_ID, USER_TOKEN, *TMS_ADDR, *TDFS_ADDR).unwrap();
    // upload train_data and target_data to mesatee's tdfs
    // let model_file_id = mesatee.upload_file(model_data_path).unwrap();
    let test_file_id = mesatee.upload_file(test_data_path).unwrap();

    let file_ids: [&str; 2] = [model_file_id, test_file_id.as_str()];

    let task = mesatee
        .create_task_with_files(FUNCTION_PREDICT_NAME, &file_ids)
        .unwrap();
    let result = task.invoke().unwrap();

    println!("result:\n{}", result);
    fs::write(result_saving_path, result.as_bytes()).unwrap();
}
