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

struct UserData {
    user_id: String,
    user_password: String,
    input_url: String,
    input_label: String,
    output_url: String,
    output_label: String,
    input_cmac: Vec<u8>,
    key: Vec<u8>,
    peer_id: String,
    peer_input_label: String,
    peer_output_label: String,
}

struct Client {
    client: teaclave_client_sdk::FrontendClient,
    user_data: UserData,
}

struct PlatformAdmin {
    client: teaclave_client_sdk::AuthenticationClient,
}

impl PlatformAdmin {
    fn new(admin_user_id: &str, admin_user_password: &str) -> Result<Self> {
        let enclave_info = teaclave_client_sdk::EnclaveInfo::from_file(ENCLAVE_INFO_PATH)?;
        let bytes = fs::read(AS_ROOT_CA_CERT_PATH)?;
        let as_root_ca_cert = pem::parse(bytes)?.contents;
        let mut client = teaclave_client_sdk::AuthenticationService::connect(
            "localhost:7776",
            &enclave_info,
            &as_root_ca_cert,
        )?;
        let token = client.user_login(admin_user_id, admin_user_password)?;
        client.set_credential(admin_user_id, &token);
        Ok(Self { client })
    }

    fn register_user(
        &mut self,
        user_id: &str,
        user_password: &str,
        role: &str,
        attribute: &str,
    ) -> Result<()> {
        self.client
            .user_register(user_id, user_password, role, attribute)
    }
}

impl Client {
    fn new(user_data: UserData) -> Result<Client> {
        let enclave_info = teaclave_client_sdk::EnclaveInfo::from_file(ENCLAVE_INFO_PATH)?;
        let bytes = fs::read(AS_ROOT_CA_CERT_PATH)?;
        let as_root_ca_cert = pem::parse(bytes)?.contents;
        let mut client = teaclave_client_sdk::AuthenticationService::connect(
            "localhost:7776",
            &enclave_info,
            &as_root_ca_cert,
        )?;

        println!("[+] {} login", user_data.user_id);
        client.user_login(&user_data.user_id, &user_data.user_password)?;

        let token = client.user_login(&user_data.user_id, &user_data.user_password)?;

        let mut client = teaclave_client_sdk::FrontendService::connect(
            "localhost:7777",
            &enclave_info,
            &as_root_ca_cert,
        )?;
        client.set_credential(&user_data.user_id, &token);

        Ok(Client { client, user_data })
    }

    fn set_task(&mut self) -> Result<String> {
        println!("[+] {} registering function", self.user_data.user_id);
        let function_id = self.client.register_function(
            "builtin-ordered-set-intersect",
            "Native Private Set Intersection.",
            "builtin",
            None,
            Some(&["order", "save_log"]),
            Some(vec![
                teaclave_client_sdk::FunctionInput::new("input_data1", "Client 0 data.", false),
                teaclave_client_sdk::FunctionInput::new("input_data2", "Client 1 data.", false),
            ]),
            Some(vec![
                teaclave_client_sdk::FunctionOutput::new("output_result1", "Output data.", false),
                teaclave_client_sdk::FunctionOutput::new("output_result2", "Output data.", false),
            ]),
        )?;
        self.client.get_function(&function_id)?;
        let function_arguments = hashmap!("order" => "ascending", "save_log" => "true"); // Order can be ascending or desending
        let inputs_ownership = hashmap!(&self.user_data.input_label => vec![self.user_data.user_id.to_string()], &self.user_data.peer_input_label => vec![self.user_data.peer_id.to_string()]);
        let outputs_ownership = hashmap!(&self.user_data.output_label => vec![self.user_data.user_id.to_string()], &self.user_data.peer_output_label => vec![self.user_data.peer_id.to_string()]);

        println!("[+] {} creating task", self.user_data.user_id);
        let task_id = self.client.create_task(
            &function_id,
            Some(function_arguments),
            "builtin",
            Some(inputs_ownership),
            Some(outputs_ownership),
        )?;
        Ok(task_id.to_string())
    }

    fn register_data(&mut self, task_id: &str) -> Result<()> {
        println!(
            "[+] {} registering input file {}",
            self.user_data.user_id, self.user_data.input_url
        );
        let data_id = self.client.register_input_file(
            &self.user_data.input_url,
            &self.user_data.input_cmac,
            teaclave_client_sdk::FileCrypto::new(
                "teaclave-file-128",
                &self.user_data.key,
                &Vec::new(),
            )?,
        )?;
        let inputs = hashmap!(&self.user_data.input_label => data_id);

        println!(
            "[+] {} registering output file {}",
            self.user_data.user_id, self.user_data.output_url
        );
        let data_id = self.client.register_output_file(
            &self.user_data.output_url,
            teaclave_client_sdk::FileCrypto::new(
                "teaclave-file-128",
                &self.user_data.key,
                &Vec::new(),
            )?,
        )?;

        let outputs = hashmap!(&self.user_data.output_label => data_id);

        println!("[+] {} assigning data to task", self.user_data.user_id);
        self.client
            .assign_data(&task_id, Some(inputs), Some(outputs))?;
        Ok(())
    }

    fn run_task(&mut self, task_id: &str) -> Result<()> {
        println!("[+] {} invoking task", self.user_data.user_id);
        self.client.invoke_task(&task_id)?;
        Ok(())
    }

    fn approve_task(&mut self, task_id: &str) -> Result<()> {
        println!("[+] {} approving task", self.user_data.user_id);
        self.client.approve_task(&task_id)?;
        Ok(())
    }

    fn get_task_result(&mut self, task_id: &str) -> Result<(Vec<u8>, Vec<String>)> {
        println!("[+] {} getting result", self.user_data.user_id);
        let response = self.client.get_task_result(&task_id)?;
        Ok(response)
    }
}

fn main() -> Result<()> {
    let mut admin = PlatformAdmin::new("admin", "teaclave")?;
    // Ignore registering errors
    let _ = admin.register_user("user0", "password", "PlatformAdmin", "");
    let _ = admin.register_user("user1", "password", "PlatformAdmin", "");
    let user0_data = UserData {
        user_id: "user0".to_string(),
        user_password: "password".to_string(),
        input_url: "http://localhost:6789/fixtures/functions/ordered_set_intersect/psi0.txt.enc"
            .to_string(),
        input_label: "input_data1".to_string(),
        output_url:
            "http://localhost:6789/fixtures/functions/ordered_set_intersect/output_psi0.enc"
                .to_string(),
        output_label: "output_result1".to_string(),
        input_cmac: vec![
            0x92, 0xf6, 0x86, 0xd4, 0xac, 0x2b, 0xa6, 0xb4, 0xff, 0xd9, 0x3b, 0xc7, 0xac, 0x5d,
            0xbf, 0x58,
        ],
        key: vec![0; 16],
        peer_id: "user1".to_string(),
        peer_input_label: "input_data2".to_string(),
        peer_output_label: "output_result2".to_string(),
    };

    let user1_data = UserData {
        user_id: "user1".to_string(),
        user_password: "password".to_string(),
        input_url: "http://localhost:6789/fixtures/functions/ordered_set_intersect/psi1.txt.enc"
            .to_string(),
        input_label: "input_data2".to_string(),
        output_url:
            "http://localhost:6789/fixtures/functions/ordered_set_intersect/output_psi1.enc"
                .to_string(),
        output_label: "output_result2".to_string(),
        input_cmac: vec![
            0x8b, 0x31, 0x04, 0x97, 0x2a, 0x6f, 0x0d, 0xe9, 0x49, 0x31, 0x5e, 0x0b, 0x45, 0xd5,
            0xdd, 0x66,
        ],
        key: vec![0; 16],
        peer_id: "user0".to_string(),
        peer_input_label: "input_data1".to_string(),
        peer_output_label: "output_result1".to_string(),
    };

    let mut user0 = Client::new(user0_data)?;
    let mut user1 = Client::new(user1_data)?;

    let task_id = user0.set_task()?;

    user0.register_data(&task_id)?;
    user1.register_data(&task_id)?;

    user0.approve_task(&task_id)?;
    user1.approve_task(&task_id)?;

    user0.run_task(&task_id)?;

    let result_user0 = user0.get_task_result(&task_id)?;

    println!(
        "[+] User 0 result: {:?} log: {:?} ",
        String::from_utf8(result_user0.0),
        result_user0.1
    );

    let result_user1 = user1.get_task_result(&task_id)?;

    println!(
        "[+] User 1 result: {:?} log {:?}",
        String::from_utf8(result_user1.0),
        result_user1.1
    );

    println!("[+] done");
    Ok(())
}
