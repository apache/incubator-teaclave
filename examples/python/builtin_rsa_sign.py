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

from teaclave import FunctionInput, FunctionOutput, FunctionArgument, OwnerList, DataMap
from utils import USER_ID, USER_PASSWORD, connect_authentication_service, connect_frontend_service, PlatformAdmin


def get_client(user_id, user_password):
    with connect_authentication_service() as client:
        print(f"[+] login")
        token = client.user_login(user_id, user_password)
    client = connect_frontend_service()
    metadata = {"id": user_id, "token": token}
    client.metadata = metadata
    return client


def register_input_file(client):
    """
    Commands when encrypting input files:
        ./teaclave_cli encrypt
        --algorithm teaclave-file-128
        --input-file ./tests/fixtures/functions/rsa_sign/key.der
        --key 00000000000000000000000000000000
        --output-file ./tests/fixtures/functions/rsa_sign/rsakey.enc
        --print-cmac
    """
    url = "http://localhost:6789/fixtures/functions/rsa_sign/rsakey.enc"
    cmac = [
        0x62, 0x0c, 0x16, 0xab, 0x58, 0x8c, 0xa6, 0x38, 0xf3, 0xb0, 0xa2, 0x69,
        0x2a, 0xb0, 0x78, 0xcf
    ]
    schema = "teaclave-file-128"
    key = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    iv = []
    key_data_id = client.register_input_file(url, schema, key, iv, cmac)

    return key_data_id


def register_func(client):
    function_id = client.register_function(
        name="builtin-rsa-sign",
        description="Native Rsa Signing Function",
        executor_type="builtin",
        arguments=[FunctionArgument("data")],
        inputs=[FunctionInput("rsa_key", "Input key file.")])

    return function_id


def create_task(client, function_id, input_file_user):
    task_id = client.create_task(
        function_id=function_id,
        executor="builtin",
        function_arguments=({
            "data": "test data",
        }),
        inputs_ownership=[OwnerList("rsa_key", [input_file_user])])

    return task_id


def rsa_task():
    platform_admin = PlatformAdmin("admin", "teaclave")
    try:
        platform_admin.register_user("rsa_sign_user1", "password1")
        platform_admin.register_user("rsa_sign_user2", "password2")
    except Exception:
        pass

    client1 = get_client("rsa_sign_user1", "password1")
    client2 = get_client("rsa_sign_user2", "password2")

    print("[+] registering key file")
    key_id = register_input_file(client1)
    print("[+] key file id" + key_id)

    print("[+] registering function")
    function_id = register_func(client2)
    print("[+] function id" + function_id)

    print("[+] creating task")
    task_id = create_task(client2, function_id, "rsa_sign_user1")
    print("[+] task id" + task_id)

    print("[+] assigning data to task")
    client1.assign_data_to_task(task_id, [DataMap("rsa_key", key_id)], [])

    print("[+] user1 approving task")
    client1.approve_task(task_id)

    print("[+] user2 approving task")
    client2.approve_task(task_id)

    print("[+] invoking task")
    client2.invoke_task(task_id)

    print("[+] getting result")
    result = client2.get_task_result(task_id)
    print("[+] done")
    client1.close()
    client2.close()

    return bytes(result)


def main():
    rt = rsa_task()
    print("[+] function return: ", rt)


if __name__ == '__main__':
    main()
