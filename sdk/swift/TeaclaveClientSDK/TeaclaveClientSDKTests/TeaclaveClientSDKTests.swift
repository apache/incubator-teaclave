//
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
//

import XCTest
@testable import TeaclaveClientSDK

class TeaclaveClientSDKTests: XCTestCase {
    let authentication_service_address = "teaclave-9900:7776"
    let frontend_service_address = "teaclave-9900:7777"

    // Setup enclave info path e.g., /incubator-teaclave/enclave_info.toml
    let enclave_info_path = ""
    // Setup AS CA certificate path e.g., /incubator-teaclave/keys/ias_root_ca_cert.pem
    let as_root_ca_cert_path = ""

    func testBuiltinEcho() throws {
        let client = AuthenticationClient(
            address: authentication_service_address,
            enclave_info_path: enclave_info_path,
            as_root_ca_cert_path: as_root_ca_cert_path
        )
        let _ = client.register(id: "test_id", password: "test_password")
        let token = try client.login(id: "test_id", password: "test_password").get()

        let register_function_request = RegisterFunctionRequest(
            name: "builtin-echo",
            description: "Native Echo Function",
            executor_type: "builtin",
            payload: [],
            arguments: ["message"],
            inputs: [],
            outputs: []
        )

        let frontend_client = FrontendClient(
            address: frontend_service_address,
            enclave_info_path: enclave_info_path,
            as_root_ca_cert_path: as_root_ca_cert_path
        )
        try frontend_client.set_credential(id: "test_id", token: token).get()
        let response = try frontend_client.register_function(with: register_function_request).get()
        let function_id = response.function_id

        let create_task_request = CreateTaskRequest(
            function_id: function_id,
            function_arguments: "{\"message\": \"Hello, Teaclave!\"}",
            executor: "builtin",
            inputs_ownership: [],
            outputs_ownership: []
        )

        let task_id = try frontend_client.create_task(with: create_task_request).get().task_id

        try frontend_client.invoke_task(task_id: task_id).get()
        let task_result = try frontend_client.get_task_result(task_id: task_id).get()
        XCTAssert(task_result == "Hello, Teaclave!")
    }

    func testBuiltinPasswordCheck() throws {
        let user0_client = AuthenticationClient(
            address: authentication_service_address,
            enclave_info_path: enclave_info_path,
            as_root_ca_cert_path: as_root_ca_cert_path
        )
        let _ = user0_client.register(id: "user0", password: "password")
        let user0_token = try user0_client.login(id: "user0", password: "password").get()

        let user0_frontend_client = FrontendClient(
            address: frontend_service_address,
            enclave_info_path: enclave_info_path,
            as_root_ca_cert_path: as_root_ca_cert_path
        )
        try user0_frontend_client.set_credential(id: "user0", token: user0_token).get()

        let register_function_request = RegisterFunctionRequest(
            name: "builtin-password-check",
            description: "Check whether a password is exposed.",
            executor_type: "builtin",
            payload: [],
            arguments: [],
            inputs: [
                ["name": "password", "description": "Client 0's data."],
                ["name": "exposed_passwords", "description": "Client 1's data."]
            ],
            outputs: []
        )
        let response = try user0_frontend_client.register_function(with: register_function_request).get()
        let function_id = response.function_id

        let create_task_request = CreateTaskRequest(
            function_id: function_id,
            function_arguments: "{}",
            executor: "builtin",
            inputs_ownership: [
                OwnerList(data_name: "password", uids: ["user0"]),
                OwnerList(data_name: "exposed_passwords", uids: ["user1"]),
            ],
            outputs_ownership: []
        )
        let task_id = try user0_frontend_client.create_task(with: create_task_request).get().task_id

        let iv = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        let key = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        let register_input_file_request = RegisterInputFileRequest(
            url: "data:text/plain;base64,c+mpvRfZ0fboR0j3rTgOGDBiubSzlCt9",
            cmac: "e84748f7ad380e183062b9b4b3942b7d",
            crypto_info: CryptoInfo(schema: "aes-gcm-128", key: key, iv: iv)
        )
        let user0_data_id = try user0_frontend_client.register_input_file(with: register_input_file_request).get().data_id


        let user0_assign_data_request = AssignDataRequest(
            task_id: task_id,
            inputs: [DataMap(data_name: "password", data_id: user0_data_id)],
            outputs: []
        )
        try user0_frontend_client.assign_data(with: user0_assign_data_request).get()

        let user1_client = AuthenticationClient(
            address: authentication_service_address,
            enclave_info_path: enclave_info_path,
            as_root_ca_cert_path: as_root_ca_cert_path
        )

        let _ = user1_client.register(id: "user1", password: "password")
        let user1_token = try user1_client.login(id: "user1", password: "password").get()
        let user1_frontend_client = FrontendClient(
            address: frontend_service_address,
            enclave_info_path: enclave_info_path,
            as_root_ca_cert_path: as_root_ca_cert_path
        )
        try user1_frontend_client.set_credential(id: "user1", token: user1_token).get()

        let user1_register_input_file_request = RegisterInputFileRequest(
            url: "http://teaclave-file-service:6789/fixtures/functions/password_check/exposed_passwords.txt.enc",
            cmac: "42b16c29edeb9ee0e4d219f3b5395946",
            crypto_info: CryptoInfo(schema: "teaclave-file-128", key: key, iv: [])
        )
        let user1_data_id = try user1_frontend_client.register_input_file(with: user1_register_input_file_request).get().data_id
        let user1_assign_data_request = AssignDataRequest(
            task_id: task_id,
            inputs: [DataMap(data_name: "exposed_passwords", data_id: user1_data_id)],
            outputs: []
        )
        try user1_frontend_client.assign_data(with: user1_assign_data_request).get()



        try user0_frontend_client.approve_task(task_id: task_id).get()
        try user1_frontend_client.approve_task(task_id: task_id).get()

        try user0_frontend_client.invoke_task(task_id: task_id).get()

        let result = try user0_frontend_client.get_task_result(task_id: task_id).get()
        XCTAssert(result == "true")

    }
}
