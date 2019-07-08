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

static FUNCTION_NAME: &'static str = "gbdt_predict";
lazy_static! {
    static ref TMS_ADDR: SocketAddr = "127.0.0.1:5554".parse().unwrap();
    static ref TDFS_ADDR: SocketAddr = "127.0.0.1:5065".parse().unwrap();
}

fn print_usage() {
    let msg = "
    ./ml_predict test_data_path model_data_path
    test data format:
        f32,f32,f32,f32 ...
        f32,f32,f32,f32 ...
        ...
    supported model:
        model saved by gbdt-rs
    output:
        f32
        f32
        ...
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
    let enclave_info_file_path = "../out/enclave_info.txt";

    let mesatee_enclave_info = MesateeEnclaveInfo::load(auditors, enclave_info_file_path).unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        print_usage();
        return;
    }
    let test_data_path = &args[1];
    let model_data_path = &args[2];

    let mesatee = Mesatee::new(
        &mesatee_enclave_info,
        "uid1",
        "token1",
        *TMS_ADDR,
        *TDFS_ADDR,
    )
    .unwrap();
    let file_id = mesatee.upload_file(model_data_path).unwrap();

    let payload_bytes = fs::read(&test_data_path).unwrap();
    let payload_str = String::from_utf8(payload_bytes).unwrap();

    let task = mesatee
        .create_task_with_files(FUNCTION_NAME, &[file_id.as_str()])
        .unwrap();
    let result = task.invoke_with_payload(&payload_str).unwrap();

    println!("result: \n{}", result);
}
