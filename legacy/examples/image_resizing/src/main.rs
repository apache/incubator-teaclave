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

use base64;
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

fn print_usage() {
    let msg = "
    ./image_resizing image_path width height filter output_format output_path

    filter: Nearest Triangle CatmullRom Gaussian Lanczos3. Default: Nearest
    output_format: PNG JPEG GIF ICO BMP. Default: JPEG

    example: ./image_resizing ./logo.png 100 100 Nearest JPEG logo.jpg
    ";
    println!("usage: \n{}", msg);
}

#[derive(Serialize)]
pub(crate) struct ImageResizePayload {
    nwidth: u32,
    nheight: u32,
    filter_type: String, //"Nearest", "Triangle", "CatmullRom", "Gaussian", "Lanczos3"
    output_format: String, //"PNG", "JPEG", , "GIF", "ICO", "BMP"
    base64_image: String,
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
    let enclave_info_file_path = "../services/enclave_info.toml";

    let mesatee_enclave_info = MesateeEnclaveInfo::load(auditors, enclave_info_file_path).unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() != 7 {
        print_usage();
        return;
    }

    let image_path = args[1].to_owned();
    let width: u32 = args[2].parse().unwrap();
    let height: u32 = args[3].parse().unwrap();
    let filter = args[4].to_owned();
    let output_format = args[5].to_owned();
    let output_path = args[6].to_owned();

    let image_bytes = fs::read(&image_path).unwrap();
    let base64_image = base64::encode(&image_bytes);

    let input_payload = ImageResizePayload {
        nwidth: width,
        nheight: height,
        filter_type: filter,
        output_format,
        base64_image,
    };

    let input_string = serde_json::to_string(&input_payload).unwrap();

    let mesatee = Mesatee::new(
        &mesatee_enclave_info,
        "uid1",
        "token1",
        *TMS_ADDR,
        *TDFS_ADDR,
    )
    .unwrap();
    let task = mesatee.create_task("image_resize").unwrap();
    let ret = task.invoke_with_payload(&input_string).unwrap();

    let output_bytes = base64::decode(&ret).unwrap();
    fs::write(output_path, &output_bytes).unwrap();
}
