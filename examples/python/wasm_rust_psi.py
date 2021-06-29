#!/usr/bin/env python3

# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

import sys

from teaclave import (AuthenticationService, FrontendService, FunctionInput,
                      FunctionOutput, OwnerList)
from utils import (AUTHENTICATION_SERVICE_ADDRESS, FRONTEND_SERVICE_ADDRESS,
                   AS_ROOT_CA_CERT_PATH, ENCLAVE_INFO_PATH, USER_ID,
                   USER_PASSWORD)


class UserData:
    def __init__(self,
                 user_id,
                 password,
                 input_url="",
                 output_url="",
                 input_cmac=[],
                 input_fid="",
                 output_fid="",
                 key=[]):
        self.user_id = user_id
        self.password = password
        self.input_url = input_url
        self.output_url = output_url
        self.input_cmac = input_cmac
        self.input_fid = input_fid
        self.output_fid = output_fid
        self.key = key


# If you're using `docker-compose` to start the Teaclave server containers,
# please change `localhost` to `teaclave-file-service`
INPUT_FILE_URL_PREFIX = "http://localhost:6789/fixtures/functions/wamr_rust_psi/"
OUTPUT_FILE_URL_PREFIX = "http://localhost:6789/fixtures/functions/wamr_rust_psi/"

USER_DATA_0 = UserData("user0", "password",
                       INPUT_FILE_URL_PREFIX + "psi0.txt.enc",
                       OUTPUT_FILE_URL_PREFIX + "output_psi0.enc", [
                           0x69, 0x3d, 0xba, 0x77, 0x03, 0x85, 0xba, 0x99,
                           0xa1, 0xb0, 0xb7, 0x09, 0x9d, 0x7f, 0x2f, 0x94
                       ], "input_data1", "output_result1",
                       [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])

USER_DATA_1 = UserData("user1", "password",
                       INPUT_FILE_URL_PREFIX + "psi1.txt.enc",
                       OUTPUT_FILE_URL_PREFIX + "output_psi1.enc", [
                           0x2b, 0x19, 0x06, 0x02, 0xb4, 0x1f, 0x9f, 0x7c,
                           0x58, 0xcb, 0x84, 0x84, 0x37, 0x9c, 0xb9, 0x11
                       ], "input_data2", "output_result2",
                       [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])


class DataList:
    def __init__(self, data_name, data_id):
        self.data_name = data_name
        self.data_id = data_id


class Client:
    def __init__(self, user_id, user_password):
        self.user_id = user_id
        self.user_password = user_password
        self.client = AuthenticationService(
            AUTHENTICATION_SERVICE_ADDRESS, AS_ROOT_CA_CERT_PATH,
            ENCLAVE_INFO_PATH).connect().get_client()
        print(f"[+] {self.user_id} registering user")
        self.client.user_register(self.user_id, self.user_password)
        print(f"[+] {self.user_id} login")
        token = self.client.user_login(self.user_id, self.user_password)
        self.client = FrontendService(
            FRONTEND_SERVICE_ADDRESS, AS_ROOT_CA_CERT_PATH,
            ENCLAVE_INFO_PATH).connect().get_client()
        metadata = {"id": self.user_id, "token": token}
        self.client.metadata = metadata

    def set_task(
        self,
        payload_file="wasm_rust_psi_payload/target/wasm32-unknown-unknown/release/wasm_rust_psi_payload.wasm"
    ):
        client = self.client

        print(f"[+] {self.user_id} registering function")

        with open(payload_file, "rb") as f:
            payload = f.read()

        function_id = client.register_function(
            name="rust-wasm-ordered-set-intersect",
            description="WAMR Rust Private Set Intersection",
            payload=list(payload),
            executor_type="wamr",
            arguments=[
                "input1_fid", "input2_fid", "output1_fid", "output2_fid"
            ],
            inputs=[
                FunctionInput(USER_DATA_0.input_fid, "Client 0 data."),
                FunctionInput(USER_DATA_1.input_fid, "Client 1 data.")
            ],
            outputs=[
                FunctionOutput(USER_DATA_0.output_fid, "Output data."),
                FunctionOutput(USER_DATA_1.output_fid, "Output data.")
            ])

        print(f"[+] {self.user_id} creating task")
        task_id = client.create_task(function_id=function_id,
                                     function_arguments=({
                                         "input1_fid":
                                         USER_DATA_0.input_fid,
                                         "input2_fid":
                                         USER_DATA_1.input_fid,
                                         "output1_fid":
                                         USER_DATA_0.output_fid,
                                         "output2_fid":
                                         USER_DATA_1.output_fid
                                     }),
                                     executor="wamr",
                                     inputs_ownership=[
                                         OwnerList(USER_DATA_0.input_fid,
                                                   [USER_DATA_0.user_id]),
                                         OwnerList(USER_DATA_1.input_fid,
                                                   [USER_DATA_1.user_id])
                                     ],
                                     outputs_ownership=[
                                         OwnerList(USER_DATA_0.output_fid,
                                                   [USER_DATA_0.user_id]),
                                         OwnerList(USER_DATA_1.output_fid,
                                                   [USER_DATA_1.user_id])
                                     ])

        return task_id

    def run_task(self, task_id):
        client = self.client
        print(f"[+] {self.user_id} invoking task")
        client.invoke_task(task_id)

    def register_data(self, task_id, input_url, input_cmac, output_url,
                      file_key, input_label, output_label):
        client = self.client

        print(f"[+] {self.user_id} registering input file")
        url = input_url
        cmac = input_cmac
        schema = "teaclave-file-128"
        key = file_key
        iv = []
        input_id = client.register_input_file(url, schema, key, iv, cmac)
        print(f"[+] {self.user_id} registering output file")
        url = output_url
        schema = "teaclave-file-128"
        key = file_key
        iv = []
        output_id = client.register_output_file(url, schema, key, iv)

        print(f"[+] {self.user_id} assigning data to task")
        client.assign_data_to_task(task_id, [DataList(input_label, input_id)],
                                   [DataList(output_label, output_id)])

    def approve_task(self, task_id):
        client = self.client
        print(f"[+] {self.user_id} approving task")
        client.approve_task(task_id)

    def get_task_result(self, task_id):
        client = self.client
        print(f"[+] {self.user_id} getting task result")
        return bytes(client.get_task_result(task_id))


def main():
    user0 = Client(USER_DATA_0.user_id, USER_DATA_0.password)
    user1 = Client(USER_DATA_1.user_id, USER_DATA_1.password)

    task_id = user0.set_task()

    user0.register_data(task_id, USER_DATA_0.input_url, USER_DATA_0.input_cmac,
                        USER_DATA_0.output_url, USER_DATA_0.key,
                        USER_DATA_0.input_fid, USER_DATA_0.output_fid)

    user1.register_data(task_id, USER_DATA_1.input_url, USER_DATA_1.input_cmac,
                        USER_DATA_1.output_url, USER_DATA_1.key,
                        USER_DATA_1.input_fid, USER_DATA_1.output_fid)

    user0.approve_task(task_id)
    user1.approve_task(task_id)

    # USER 0 start the computation
    user0.run_task(task_id)

    # USER 0, 1 get the result
    result_user0 = user0.get_task_result(task_id)
    result_user1 = user1.get_task_result(task_id)

    print("[+] User 0 result: " + result_user0.decode("utf-8"))
    print("[+] User 1 result: " + result_user1.decode("utf-8"))


if __name__ == '__main__':
    main()
