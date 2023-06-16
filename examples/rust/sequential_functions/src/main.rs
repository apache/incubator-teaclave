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
use teaclave_client_sdk::{
    AuthenticationClient, AuthenticationService, EnclaveInfo, FileCrypto, FrontendClient,
    FrontendService, FunctionArgument, FunctionInput, FunctionOutput,
};

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
const AS_ROOT_CA_CERT_PATH: &str = "../../../config/keys/dcap_root_ca_cert.pem";
#[cfg(not(dcap))]
const AS_ROOT_CA_CERT_PATH: &str = "../../../config/keys/ias_root_ca_cert.pem";

const JOIN_INPUT_LABEL1: &str = "input_data1";
const JOIN_INPUT_LABEL2: &str = "input_data2";
const JOIN_OUTPUT_LABEL: &str = "output_result";
const TRAIN_INPUT_LABEL: &str = "training_data";
const TRAIN_OUTPUT_LABEL: &str = "trained_model";

struct UserData {
    user_id: String,
    user_password: String,
    input_url: String,
    input_label: String,
    output_url: String,
    input_cmac: Vec<u8>,
    key: Vec<u8>,
    peer_id: String,
    peer_input_label: String,
}

struct Client {
    client: FrontendClient,
    user_data: UserData,
}

struct PlatformAdmin {
    client: AuthenticationClient,
}

impl PlatformAdmin {
    fn new(admin_user_id: &str, admin_user_password: &str) -> Result<Self> {
        let enclave_info = EnclaveInfo::from_file(ENCLAVE_INFO_PATH)?;
        let bytes = fs::read(AS_ROOT_CA_CERT_PATH)?;
        let as_root_ca_cert = pem::parse(bytes)?.contents;
        let mut client = AuthenticationService::connect(
            "https://localhost:7776",
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
        let enclave_info = EnclaveInfo::from_file(ENCLAVE_INFO_PATH)?;
        let bytes = fs::read(AS_ROOT_CA_CERT_PATH)?;
        let as_root_ca_cert = pem::parse(bytes)?.contents;
        let mut client = AuthenticationService::connect(
            "https://localhost:7776",
            &enclave_info,
            &as_root_ca_cert,
        )?;

        println!("[+] {} login", user_data.user_id);
        client.user_login(&user_data.user_id, &user_data.user_password)?;

        let token = client.user_login(&user_data.user_id, &user_data.user_password)?;

        let mut client =
            FrontendService::connect("https://localhost:7777", &enclave_info, &as_root_ca_cert)?;
        client.set_credential(&user_data.user_id, &token);

        Ok(Client { client, user_data })
    }

    fn set_train_task(&mut self) -> Result<String> {
        println!("[+] {} registering function", self.user_data.user_id);
        let function_id = self.client.register_function(
            "builtin-gbdt-train",
            "Native Gbdt Training Function.",
            "builtin",
            None,
            Some(vec![
                FunctionArgument::new("feature_size", "", true),
                FunctionArgument::new("max_depth", "", true),
                FunctionArgument::new("iterations", "", true),
                FunctionArgument::new("shrinkage", "", true),
                FunctionArgument::new("feature_sample_ratio", "", true),
                FunctionArgument::new("data_sample_ratio", "", true),
                FunctionArgument::new("min_leaf_size", "", true),
                FunctionArgument::new("training_optimization_level", "", true),
                FunctionArgument::new("loss", "", true),
            ]),
            Some(vec![FunctionInput::new(
                TRAIN_INPUT_LABEL,
                "Fusion data.",
                false,
            )]),
            Some(vec![FunctionOutput::new(
                TRAIN_OUTPUT_LABEL,
                "Output trained model.",
                false,
            )]),
            None,
        )?;
        self.client.get_function(&function_id)?;

        let function_arguments = hashmap!(
                                "feature_size" => 4,
                                "max_depth" => 4,
                                "iterations" => 100,
                                "shrinkage" => 0.1,
                                "feature_sample_ratio" => 1.0,
                                "data_sample_ratio" => 1.0,
                                "min_leaf_size" => 1,
                                "loss" => "LAD",
                                "training_optimization_level" => 2);

        let inputs_ownership = hashmap!(TRAIN_INPUT_LABEL => vec![self.user_data.user_id.to_string(),self.user_data.peer_id.to_string()]);
        let outputs_ownership =
            hashmap!(TRAIN_OUTPUT_LABEL =>vec![self.user_data.user_id.to_string()],);

        println!("[+] {} creating task", self.user_data.user_id);
        let task_id = self.client.create_task(
            &function_id,
            Some(function_arguments),
            "builtin",
            Some(inputs_ownership),
            Some(outputs_ownership),
        )?;
        Ok(task_id)
    }

    fn set_fusion_task(&mut self) -> Result<String> {
        println!("[+] {} registering function", self.user_data.user_id);
        let function_id = self.client.register_function(
            "builtin-ordered-set-join",
            "Join two sets of CSV data based on the specified sorted columns.",
            "builtin",
            None,
            Some(vec![
                FunctionArgument::new("left_column", "", true),
                FunctionArgument::new("right_column", "", true),
                FunctionArgument::new("ascending", "true", true),
                FunctionArgument::new("drop", "true", true),
                FunctionArgument::new("save_log", "false", true),
            ]),
            Some(vec![
                FunctionInput::new(JOIN_INPUT_LABEL1, "Client 0 data.", false),
                FunctionInput::new(JOIN_INPUT_LABEL2, "Client 1 data.", false),
            ]),
            Some(vec![FunctionOutput::new(
                JOIN_OUTPUT_LABEL,
                "Output data.",
                false,
            )]),
            None,
        )?;
        self.client.get_function(&function_id)?;
        let function_arguments = hashmap!("left_column" => 0, "right_column" => 0, "ascending" => true, "drop"=>true,"save_log" => "true");
        let inputs_ownership = hashmap!(&self.user_data.input_label => vec![self.user_data.user_id.to_string()], &self.user_data.peer_input_label => vec![self.user_data.peer_id.to_string()]);
        let outputs_ownership = hashmap!(JOIN_OUTPUT_LABEL=>vec![
            self.user_data.user_id.to_string(),
            self.user_data.peer_id.to_string(),
        ]);

        println!("[+] {} creating task", self.user_data.user_id);
        let task_id = self.client.create_task(
            &function_id,
            Some(function_arguments),
            "builtin",
            Some(inputs_ownership),
            Some(outputs_ownership),
        )?;
        Ok(task_id)
    }

    fn register_input_data(&mut self, task_id: &str) -> Result<()> {
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
        self.client.assign_data(task_id, Some(inputs), None)?;

        Ok(())
    }

    fn register_input_from_output(
        &mut self,
        task_id: &str,
        label: &str,
        data_id: &str,
    ) -> Result<()> {
        let new_id = self.client.register_input_from_output(data_id)?;
        let inputs = hashmap!(label => new_id);
        self.client.assign_data(task_id, Some(inputs), None)
    }

    fn register_output_data(&mut self, task_id: &str, label: &str) -> Result<()> {
        let data_id = self.client.register_output_file(
            &self.user_data.output_url,
            FileCrypto::new("teaclave-file-128", &self.user_data.key, &Vec::new())?,
        )?;
        let outputs = hashmap!(label => data_id);
        self.client.assign_data(task_id, None, Some(outputs))
    }

    fn register_fusion_data(&mut self, task_id: &str, label: &str) -> Result<String> {
        let data_id = self.client.register_fusion_output(vec![
            self.user_data.user_id.to_string(),
            self.user_data.peer_id.to_string(),
        ])?;

        let outputs = hashmap!(label => data_id.clone());

        println!(
            "[+] {} assigning fusion data to task",
            self.user_data.user_id
        );
        self.client.assign_data(task_id, None, Some(outputs))?;
        Ok(data_id)
    }

    fn run_task(&mut self, task_id: &str) -> Result<()> {
        println!("[+] {} invoking task", self.user_data.user_id);
        self.client.invoke_task(task_id)?;
        Ok(())
    }

    fn approve_task(&mut self, task_id: &str) -> Result<()> {
        println!("[+] {} approving task", self.user_data.user_id);
        self.client.approve_task(task_id)?;
        Ok(())
    }

    fn get_task_result(&mut self, task_id: &str) -> Result<(Vec<u8>, Vec<String>)> {
        println!("[+] {} getting result", self.user_data.user_id);
        let response = self.client.get_task_result(task_id)?;
        Ok(response)
    }
}

// User0 provides some training features, while User1 provides another set of training features and label.
// Based on the sorted ID columns, these two data are concatenated and used as training data for the GBDT.
fn main() -> Result<()> {
    let mut admin = PlatformAdmin::new("admin", "teaclave")?;
    // Ignore registering errors
    let _ = admin.register_user("user0", "password", "PlatformAdmin", "");
    let _ = admin.register_user("user1", "password", "PlatformAdmin", "");
    let user0_data = UserData {
        user_id: "user0".to_string(),
        user_password: "password".to_string(),
        input_url: "http://localhost:6789/fixtures/functions/ordered_set_join/join0.csv.enc"
            .to_string(),
        input_label: JOIN_INPUT_LABEL1.to_string(),
        output_url: "http://localhost:6789/fixtures/functions/gbdt_training/output_model.enc"
            .to_string(),
        input_cmac: vec![
            0x3f, 0x91, 0xd2, 0x74, 0x47, 0x63, 0x44, 0x5d, 0x26, 0x5e, 0xa4, 0x69, 0xde, 0xbb,
            0x74, 0xf0,
        ],
        key: vec![0; 16],
        peer_id: "user1".to_string(),
        peer_input_label: JOIN_INPUT_LABEL2.to_string(),
    };

    let user1_data = UserData {
        user_id: "user1".to_string(),
        user_password: "password".to_string(),
        input_url: "http://localhost:6789/fixtures/functions/ordered_set_join/join1.csv.enc"
            .to_string(),
        input_label: JOIN_INPUT_LABEL2.to_string(),
        output_url: "".to_string(),
        input_cmac: vec![
            0xd1, 0xe5, 0xa5, 0x20, 0x48, 0x9c, 0x93, 0xd0, 0x25, 0x4c, 0x8c, 0x22, 0xcd, 0xef,
            0xab, 0x89,
        ],
        key: vec![0; 16],
        peer_id: "user0".to_string(),
        peer_input_label: JOIN_INPUT_LABEL1.to_string(),
    };

    let mut user0 = Client::new(user0_data)?;
    let mut user1 = Client::new(user1_data)?;

    let task_id = user0.set_fusion_task()?;

    user0.register_input_data(&task_id)?;
    user1.register_input_data(&task_id)?;
    let fusion_id = user0.register_fusion_data(&task_id, JOIN_OUTPUT_LABEL)?;

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

    let train_task_id = user0.set_train_task()?;
    user0.register_input_from_output(&train_task_id, TRAIN_INPUT_LABEL, &fusion_id)?;
    user0.register_output_data(&train_task_id, TRAIN_OUTPUT_LABEL)?;
    user0.approve_task(&train_task_id)?;
    anyhow::ensure!(
        user0.run_task(&train_task_id).is_err(),
        "An error should be returned here because it is waiting for user1's approval."
    );
    user1.approve_task(&train_task_id)?;
    user0.run_task(&train_task_id)?;

    let result_user0 = user0.get_task_result(&train_task_id)?;
    println!("[+] User 0 result: {:?}", String::from_utf8(result_user0.0),);
    println!("[+] done");
    Ok(())
}
