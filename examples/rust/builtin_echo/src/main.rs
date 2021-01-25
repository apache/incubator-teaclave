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

use anyhow::Result;
use std::env;
use std::fs;
use teaclave_client_sdk;

#[macro_export]
macro_rules! hashmap {
    ($( $key: expr => $value: expr, )+) => { hashmap!($($key => $value),+) };
    ($( $key: expr => $value: expr ),*) => {{
        let mut map = ::std::collections::HashMap::new();
        $( map.insert($key.into(), $value.into()); )*
        map
    }}
}

const ENCLAVE_INFO_PATH: &str = "../../../release/services/enclave_info.toml";
#[cfg(dcap)]
const AS_ROOT_CA_CERT_PATH: &str = "../../../keys/dcap_root_ca_cert.pem";
#[cfg(not(dcap))]
const AS_ROOT_CA_CERT_PATH: &str = "../../../keys/ias_root_ca_cert.pem";
const USER_ID: &str = "echo_test_example_user";
const USER_PASSWORD: &str = "test_password";

fn echo(message: &str) -> Result<Vec<u8>> {
    let enclave_info = teaclave_client_sdk::EnclaveInfo::from_file(ENCLAVE_INFO_PATH)?;
    let bytes = fs::read(AS_ROOT_CA_CERT_PATH)?;
    let as_root_ca_cert = pem::parse(bytes)?.contents;
    let mut client = teaclave_client_sdk::AuthenticationService::connect(
        "localhost:7776",
        &enclave_info,
        &as_root_ca_cert,
    )?;

    println!("[+] registering user");
    client.user_register(USER_ID, USER_PASSWORD)?;

    println!("[+] login");
    let token = client.user_login(USER_ID, USER_PASSWORD)?;

    let mut client = teaclave_client_sdk::FrontendService::connect(
        "localhost:7777",
        &enclave_info,
        &as_root_ca_cert,
    )?;
    client.set_credential(USER_ID, &token);

    println!("[+] registering function");
    let function_id = client.register_function(
        "builtin-echo",
        "An native echo function.",
        "builtin",
        None,
        Some(&["message"]),
        None,
        None,
    )?;

    println!(
        "[+] getting registered function name {}",
        client.get_function(&function_id)?.name
    );

    let function_arguments = hashmap! {"message" => message};

    println!("[+] creating task");
    let task_id = client.create_task(
        &function_id,
        Some(function_arguments),
        "builtin",
        None,
        None,
    )?;

    println!("[+] invoking task");
    let _ = client.invoke_task(&task_id)?;

    println!("[+] getting result");
    let response = client.get_task_result(&task_id)?;
    Ok(response)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
            let message = "Hello, Teaclave!";
            println!(
                "[+] function return: {:?}",
                String::from_utf8(echo(message)?)
            );
        }
        _ => {
            let message = &args[1];
            println!(
                "[+] function return: {:?}",
                String::from_utf8(echo(message)?)
            );
        }
    }
    println!("[+] done");
    Ok(())
}
