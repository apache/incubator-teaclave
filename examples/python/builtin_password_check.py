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

from teaclave import FunctionInput, FunctionOutput, OwnerList, DataMap
from utils import USER_ID, USER_PASSWORD, connect_authentication_service, connect_frontend_service, PlatformAdmin

# In the example, user 0 creates the task and user 0, 1, upload their private data.
# Then user 0 invokes the task and user 0, 1 get the result.


class UserData:

    def __init__(self,
                 user_id,
                 password,
                 input_url="",
                 encryption_algorithm="teaclave-file-128",
                 input_cmac=[],
                 iv=[],
                 key=[]):
        self.user_id = user_id
        self.password = password
        self.input_url = input_url
        self.encryption_algorithm = encryption_algorithm
        self.input_cmac = input_cmac
        self.iv = iv
        self.key = key


INPUT_FILE_URL_PREFIX = "http://localhost:6789/fixtures/functions/password_check/"

# Client
USER_DATA_0 = UserData(
    "user0",
    "password",
    "data:text/plain;base64,c+mpvRfZ0fboR0j3rTgOGDBiubSzlCt9",  # base64 of encrypted string "password"
    "aes-gcm-128",
    [
        0xe8, 0x47, 0x48, 0xf7, 0xad, 0x38, 0x0e, 0x18, 0x30, 0x62, 0xb9, 0xb4,
        0xb3, 0x94, 0x2b, 0x7d
    ],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])

# Data provider
USER_DATA_1 = UserData("user1", "password",
                       INPUT_FILE_URL_PREFIX + "exposed_passwords.txt.enc",
                       "teaclave-file-128", [
                           0xa3, 0x1d, 0xd0, 0xb7, 0xde, 0x0b, 0x6c, 0xc7,
                           0xe0, 0xde, 0xc1, 0xdc, 0xf7, 0xa4, 0x64, 0x79
                       ], [], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])


class DataList:

    def __init__(self, data_name, data_id):
        self.data_name = data_name
        self.data_id = data_id


class Client:

    def __init__(self, user_id, user_password):
        self.user_id = user_id
        self.user_password = user_password
        with connect_authentication_service() as client:
            print(f"[+] {self.user_id} login")
            token = client.user_login(self.user_id, self.user_password)
        self.client = connect_frontend_service()
        metadata = {"id": self.user_id, "token": token}
        self.client.metadata = metadata

    def set_task(self):
        client = self.client

        print(f"[+] {self.user_id} registering function")

        function_id = client.register_function(
            name="builtin-password-check",
            description="Check whether a password is exposed.",
            executor_type="builtin",
            arguments=[],
            inputs=[
                FunctionInput("password", "Client 0 data."),
                FunctionInput("exposed_passwords", "Client 1 data.")
            ],
            outputs=[])

        print(f"[+] {self.user_id} creating task")
        task_id = client.create_task(
            function_id=function_id,
            function_arguments={},
            executor="builtin",
            inputs_ownership=[
                OwnerList("password", [USER_DATA_0.user_id]),
                OwnerList("exposed_passwords", [USER_DATA_1.user_id])
            ],
        )

        return task_id

    def run_task(self, task_id):
        client = self.client
        print(f"[+] {self.user_id} invoking task")
        client.invoke_task(task_id)

    def register_data(self, task_id, input_url, algorithm, input_cmac,
                      file_key, iv, input_label):
        client = self.client

        print(f"[+] {self.user_id} registering input file")
        url = input_url
        cmac = input_cmac
        schema = algorithm
        key = file_key
        input_id = client.register_input_file(url, schema, key, iv, cmac)

        print(f"[+] {self.user_id} assigning data to task")
        client.assign_data_to_task(task_id, [DataList(input_label, input_id)],
                                   [])

    def approve_task(self, task_id):
        client = self.client
        print(f"[+] {self.user_id} approving task")
        client.approve_task(task_id)

    def get_task_result(self, task_id):
        client = self.client
        print(f"[+] {self.user_id} getting task result")
        return bytes(client.get_task_result(task_id))


def main():
    platform_admin = PlatformAdmin("admin", "teaclave")
    try:
        platform_admin.register_user(USER_DATA_0.user_id, USER_DATA_0.password)
        platform_admin.register_user(USER_DATA_1.user_id, USER_DATA_1.password)
    except Exception:
        pass

    user0 = Client(USER_DATA_0.user_id, USER_DATA_0.password)
    user1 = Client(USER_DATA_1.user_id, USER_DATA_1.password)

    task_id = user0.set_task()

    user0.register_data(
        task_id,
        USER_DATA_0.input_url,
        USER_DATA_0.encryption_algorithm,
        USER_DATA_0.input_cmac,
        USER_DATA_0.key,
        USER_DATA_0.iv,
        "password",
    )

    user1.register_data(
        task_id,
        USER_DATA_1.input_url,
        USER_DATA_1.encryption_algorithm,
        USER_DATA_1.input_cmac,
        USER_DATA_1.key,
        USER_DATA_1.iv,
        "exposed_passwords",
    )

    user0.approve_task(task_id)
    user1.approve_task(task_id)

    ## USER 0 start the computation
    user0.run_task(task_id)

    ## USER 0, 1 get the result
    result_user0 = user0.get_task_result(task_id)
    result_user1 = user1.get_task_result(task_id)

    print("[+] User 0 result: " + result_user0.decode("utf-8"))
    print("[+] User 1 result: " + result_user1.decode("utf-8"))


if __name__ == '__main__':
    main()
