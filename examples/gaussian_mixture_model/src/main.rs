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
pub(crate) struct GmmPayload {
    input_model_columns: usize,
    input_model_data: String,
    test_data: String,
}

fn print_usage() {
    let msg = "
    ./gaussian_mixture_model input_model_data_columns input_model_data_path test_data_path 
    input_model_data format:
        f32,f32,f32,f32 ...
        f32,f32,f32,f32 ...
        ...
    test_data format:
        f32,f32,f32,f32 ...
    ";
    println!("usage: \n{}", msg);
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

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        print_usage();
        return;
    }

    let columns = args[1].parse().unwrap();
    let input_model_data_path = &args[2];
    let test_date_path = &args[3];

    let input_model_data_bytes = fs::read(&input_model_data_path).unwrap();
    let input_model_data_str = String::from_utf8(input_model_data_bytes).unwrap();

    let test_data_bytes = fs::read(&test_date_path).unwrap();
    let test_data_str = String::from_utf8(test_data_bytes).unwrap();

    let input_payload = GmmPayload {
        input_model_columns: columns,
        input_model_data: input_model_data_str,
        test_data: test_data_str,
    };

    let input_string = serde_json::to_string(&input_payload).unwrap();
    let enclave_info_file_path = "../out/enclave_info.txt";
    let mesatee_enclave_info = MesateeEnclaveInfo::load(auditors, enclave_info_file_path).unwrap();
    let mesatee = Mesatee::new(
        &mesatee_enclave_info,
        "uid1",
        "token1",
        *TMS_ADDR,
        *TDFS_ADDR,
    )
    .unwrap();
    let task = mesatee.create_task("gaussian_mixture_model").unwrap();
    let result = task.invoke_with_payload(&input_string).unwrap();

    println!("{}", result)
}
