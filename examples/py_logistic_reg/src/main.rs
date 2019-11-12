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
use std::net::SocketAddr;
use std::{env, fs};

static FUNCTION_NAME: &str = "mesapy_from_buffer";
static USER_ID: &str = "uid";
static USER_TOKEN: &str = "token";

lazy_static! {
    static ref TMS_ADDR: SocketAddr = "127.0.0.1:5554".parse().unwrap();
    static ref TDFS_ADDR: SocketAddr = "127.0.0.1:5065".parse().unwrap();
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
    let info = MesateeEnclaveInfo::load(auditors, enclave_info_file_path).unwrap();

    let args_string: Vec<String> = env::args().collect();
    let args: Vec<&str> = args_string.iter().map(|s| s.as_str()).collect();
    if args.len() < 2 {
        println!("Please specify the python script");
        return;
    }
    let py_script_path = args[1];
    let py_script = fs::read(py_script_path).unwrap();
    let request = base64::encode(&py_script);

    let mesatee = Mesatee::new(&info, USER_ID, USER_TOKEN, *TMS_ADDR, *TDFS_ADDR).unwrap();
    let train_file_id = mesatee
        .upload_file("../examples/py_logistic_reg/data/train.txt")
        .unwrap();
    let predict_file_id = mesatee
        .upload_file("../examples/py_logistic_reg/data/predict.txt")
        .unwrap();

    let file_ids: [&str; 2] = [train_file_id.as_str(), predict_file_id.as_str()];
    
    let task = mesatee
        .create_task_with_files(FUNCTION_NAME, &file_ids)
        .unwrap();
    let result = task.invoke_with_payload(&request).unwrap();
    println!("{}", result);
}
