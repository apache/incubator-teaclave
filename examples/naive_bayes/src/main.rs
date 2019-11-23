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

lazy_static! {
    static ref TMS_ADDR: SocketAddr = "127.0.0.1:5554".parse().unwrap();
    static ref TDFS_ADDR: SocketAddr = "127.0.0.1:5065".parse().unwrap();
}

#[derive(Serialize)]
pub(crate) struct NaiveBayesPayload {
    input_model_columns: usize,
    input_model_data: String,
    target_model_columns: usize,
    target_model_data: String,
}

fn print_usage() {
    let msg = "
    ./naive_bayes input_model_data_columns input_model_data_path target_model_columns target_model_data_path 
    input_model_data format:
        f32,f32,f32,f32 ...
        f32,f32,f32,f32 ...
        ...
    target_model_data format:
        1.0,0.0,0.0 ...
        1.0,0.0,0.0 ...
        0.0,1.0,0.0 ...
        ....
    ";
    println!("usage: \n{}", msg);
}

fn main() {
    let auditors = vec![
        (
            "../service/auditors/godzilla/godzilla.public.der",
            "../service/auditors/godzilla/godzilla.sign.sha256",
        ),
        (
            "../service/auditors/optimus_prime/optimus_prime.public.der",
            "../service/auditors/optimus_prime/optimus_prime.sign.sha256",
        ),
        (
            "../service/auditors/albus_dumbledore/albus_dumbledore.public.der",
            "../service/auditors/albus_dumbledore/albus_dumbledore.sign.sha256",
        ),
    ];

    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        print_usage();
        return;
    }

    let input_columns = args[1].parse().unwrap();
    let input_model_data_path = &args[2];
    let target_columns = args[3].parse().unwrap();
    let target_model_data_path = &args[4];

    let input_model_data_bytes = fs::read(&input_model_data_path).unwrap();
    let input_model_data_str = String::from_utf8(input_model_data_bytes).unwrap();

    let target_model_data_bytes = fs::read(&target_model_data_path).unwrap();
    let target_model_data_str = String::from_utf8(target_model_data_bytes).unwrap();

    let input_payload = NaiveBayesPayload {
        input_model_columns: input_columns,
        input_model_data: input_model_data_str,
        target_model_columns: target_columns,
        target_model_data: target_model_data_str,
    };

    let input_string = serde_json::to_string(&input_payload).unwrap();
    let enclave_info_file_path = "../service/enclave_info.txt";
    let mesatee_enclave_info = MesateeEnclaveInfo::load(auditors, enclave_info_file_path).unwrap();
    let mesatee = Mesatee::new(
        &mesatee_enclave_info,
        "uid1",
        "token1",
        *TMS_ADDR,
        *TDFS_ADDR,
    )
    .unwrap();
    let task = mesatee.create_task("naive_bayes").unwrap();
    let result = task.invoke_with_payload(&input_string).unwrap();

    println!("{}", result)
}
