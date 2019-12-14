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
pub(crate) struct KmeansPayload {
    k: usize,
    feature_num: usize,
    data: String,
}

fn print_usage() {
    let msg = "
    ./kmeans k feature_num test_data_path
    test data format:
        f32,f32,f32,f32 ...
        f32,f32,f32,f32 ...
        ...
    output:
    labels:
        i32
        i32
        ...
    ";
    println!("usage: \n{}", msg);
}

fn main() {
    let auditors = vec![
        (
            "../services/auditors/godzilla/godzilla.public.der",
            "../services/auditors/godzilla/godzilla.sign.sha256",
        ),
        (
            "../services/auditors/optimus_prime/optimus_prime.public.der",
            "../services/auditors/optimus_prime/optimus_prime.sign.sha256",
        ),
        (
            "../services/auditors/albus_dumbledore/albus_dumbledore.public.der",
            "../services/auditors/albus_dumbledore/albus_dumbledore.sign.sha256",
        ),
    ];

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        print_usage();
        return;
    }
    let k_number = args[1].parse().unwrap();
    let feature_number = args[2].parse().unwrap();
    let data_path = &args[3];

    let data_bytes = fs::read(&data_path).unwrap();
    let data_str = String::from_utf8(data_bytes).unwrap();

    let input_payload = KmeansPayload {
        k: k_number,
        feature_num: feature_number,
        data: data_str,
    };
    let input_string = serde_json::to_string(&input_payload).unwrap();

    let enclave_info_file_path = "../services/enclave_info.toml";
    let mesatee_enclave_info = MesateeEnclaveInfo::load(auditors, enclave_info_file_path).unwrap();

    let mesatee = Mesatee::new(
        &mesatee_enclave_info,
        "uid1",
        "token1",
        *TMS_ADDR,
        *TDFS_ADDR,
    )
    .unwrap();
    let task = mesatee.create_task("kmeans_cluster").unwrap();
    let result = task.invoke_with_payload(&input_string).unwrap();

    print!("result: \n{}", result)
}
