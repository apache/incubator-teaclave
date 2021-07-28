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
                      DataMap, OwnerList)
from utils import (AUTHENTICATION_SERVICE_ADDRESS, FRONTEND_SERVICE_ADDRESS,
                   AS_ROOT_CA_CERT_PATH, ENCLAVE_INFO_PATH, USER_ID,
                   USER_PASSWORD)

# If you're using `docker-compose` to start the Teaclave server containers,
# please change `localhost` to `teaclave-file-service`
INPUT_FILE_URL_PREFIX = "http://localhost:6789/fixtures/functions/wamr_tvm_mnist/"
INPUT_FILENAME = "img_10.jpg.enc"
INPUT_URL = INPUT_FILE_URL_PREFIX + INPUT_FILENAME
INPUT_CMAC = [
    0xcc, 0x7f, 0xd1, 0xf6, 0x76, 0xd9, 0x3c, 0xfa, 0x52, 0x63, 0x3a, 0x25,
    0xd8, 0x1c, 0xee, 0x74
]
INPUT_KEY = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
USER_ID = "user_mnist"
USER_PASSWORD = "password_mnist"
PAYLOAD_FILE = "wasm_tvm_mnist_payload/mnist.wasm"


def main():

    client = AuthenticationService(AUTHENTICATION_SERVICE_ADDRESS,
                                   AS_ROOT_CA_CERT_PATH,
                                   ENCLAVE_INFO_PATH).connect().get_client()

    print(f"[+] {USER_ID} registering user")
    client.user_register(USER_ID, USER_PASSWORD)

    print(f"[+] {USER_ID} login")
    token = client.user_login(USER_ID, USER_PASSWORD)

    client = FrontendService(FRONTEND_SERVICE_ADDRESS, AS_ROOT_CA_CERT_PATH,
                             ENCLAVE_INFO_PATH).connect().get_client()
    metadata = {"id": USER_ID, "token": token}
    client.metadata = metadata

    print(f"[+] {USER_ID} registering function")

    with open(PAYLOAD_FILE, "rb") as f:
        payload = f.read()

    function_id = client.register_function(
        name="wasm-tvm-mnist",
        description="WAMR TVM MNIST Prediction",
        payload=list(payload),
        executor_type="wamr",
        arguments=["input_img"],
        inputs=[
            FunctionInput("input_img",
                          "Input image for handwriting number perdiction")
        ],
        outputs=[])

    print(f"[+] {USER_ID} creating task")
    task_id = client.create_task(
        function_id=function_id,
        function_arguments=({
            "input_img": "input_img",
        }),
        executor="wamr",
        inputs_ownership=[OwnerList("input_img", [USER_ID])],
        outputs_ownership=[])

    print(f"[+] {USER_ID} registering input file")
    schema = "teaclave-file-128"
    input_id = client.register_input_file(INPUT_URL, schema, INPUT_KEY, [],
                                          INPUT_CMAC)

    print(f"[+] {USER_ID} assigning data to task")
    client.assign_data_to_task(task_id, [DataMap("input_img", input_id)], [])

    print(f"[+] {USER_ID} approving task")
    client.approve_task(task_id)

    print(f"[+] {USER_ID} invoking task")
    client.invoke_task(task_id)

    print(f"[+] {USER_ID} getting task result")
    result = client.get_task_result(task_id)

    print("[+] User result: " + bytes(result).decode("utf-8"))


if __name__ == '__main__':
    main()
