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

from teaclave import FunctionInput, FunctionOutput, FunctionArgument, OwnerList, DataMap
from utils import connect_authentication_service, connect_frontend_service, PlatformAdmin

# In the example, user 3 creates the task and user 0, 1, 2 upload their private data.
# Then user 3 invokes the task and user 0, 1, 2 get the result.


class UserData:

    def __init__(self,
                 user_id,
                 password,
                 input_url="",
                 output_url="",
                 input_cmac=[],
                 key=[]):
        self.user_id = user_id
        self.password = password
        self.input_url = input_url
        self.output_url = output_url
        self.input_cmac = input_cmac
        self.key = key


INPUT_FILE_URL_PREFIX = "http://localhost:6789/fixtures/functions/private_join_and_compute/three_party_data/"
OUTPUT_FILE_URL_PREFIX = "http://localhost:6789/fixtures/functions/private_join_and_compute/three_party_data/"

USER_DATA_0 = UserData("user0", "password",
                       INPUT_FILE_URL_PREFIX + "bank_a.enc",
                       OUTPUT_FILE_URL_PREFIX + "user0_output.enc", [
                           0xf6, 0xa4, 0x58, 0x5a, 0x49, 0x03, 0xf8, 0xde,
                           0xa0, 0xee, 0x90, 0xac, 0x84, 0xa5, 0x0d, 0xba
                       ], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])

USER_DATA_1 = UserData("user1", "password",
                       INPUT_FILE_URL_PREFIX + "bank_b.enc",
                       OUTPUT_FILE_URL_PREFIX + "user1_output.enc", [
                           0xab, 0x60, 0x65, 0x31, 0x4f, 0xcb, 0xac, 0xd6,
                           0x97, 0x1f, 0xf5, 0x5d, 0x38, 0x21, 0xb5, 0x1e
                       ], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])

USER_DATA_2 = UserData("user2", "password",
                       INPUT_FILE_URL_PREFIX + "bank_c.enc",
                       OUTPUT_FILE_URL_PREFIX + "user2_output.enc", [
                           0x9d, 0x7d, 0xbe, 0x46, 0x58, 0x06, 0x73, 0x1a,
                           0x43, 0x00, 0x8a, 0xb0, 0x94, 0xc9, 0x76, 0x88
                       ], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])

USER_DATA_3 = UserData("user3", "password")


class ConfigClient:

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
            name="builtin-private-join-and-compute",
            description="Native Private Join And Compute",
            executor_type="builtin",
            arguments=[FunctionArgument("num_user")],
            inputs=[
                FunctionInput("input_data0", "Bank A data file."),
                FunctionInput("input_data1", "Bank B data file."),
                FunctionInput("input_data2", "Bank C data file.")
            ],
            outputs=[
                FunctionOutput("output_data0", "Output data."),
                FunctionOutput("output_data1", "Output data."),
                FunctionOutput("output_data2", "Output date.")
            ])

        print(f"[+] {self.user_id} creating task")
        task_id = client.create_task(function_id=function_id,
                                     function_arguments=({
                                         "num_user": 3,
                                     }),
                                     executor="builtin",
                                     inputs_ownership=[
                                         OwnerList("input_data0",
                                                   [USER_DATA_0.user_id]),
                                         OwnerList("input_data1",
                                                   [USER_DATA_1.user_id]),
                                         OwnerList("input_data2",
                                                   [USER_DATA_2.user_id])
                                     ],
                                     outputs_ownership=[
                                         OwnerList("output_data0",
                                                   [USER_DATA_0.user_id]),
                                         OwnerList("output_data1",
                                                   [USER_DATA_1.user_id]),
                                         OwnerList("output_data2",
                                                   [USER_DATA_2.user_id])
                                     ])

        return task_id

    def run_task(self, task_id):
        client = self.client
        client.approve_task(task_id)
        print(f"[+] {self.user_id} invoking task")
        client.invoke_task(task_id)


class DataClient:

    def __init__(self, user_id, user_password):
        self.user_id = user_id
        self.user_password = user_password
        with connect_authentication_service() as client:
            print(f"[+] {self.user_id} login")
            token = client.user_login(self.user_id, self.user_password)
        self.client = connect_frontend_service()
        metadata = {"id": self.user_id, "token": token}
        self.client.metadata = metadata

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
        client.assign_data_to_task(task_id, [DataMap(input_label, input_id)],
                                   [DataMap(output_label, output_id)])

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
        platform_admin.register_user(USER_DATA_2.user_id, USER_DATA_2.password)
        platform_admin.register_user(USER_DATA_3.user_id, USER_DATA_3.password)
    except Exception:
        pass

    ## USER 3 creates the task
    config_client = ConfigClient(USER_DATA_3.user_id, USER_DATA_3.password)
    task_id = config_client.set_task()

    ## USER 0, 1, 2 join the task and upload their data
    user0 = DataClient(USER_DATA_0.user_id, USER_DATA_0.password)
    user0.register_data(task_id, USER_DATA_0.input_url, USER_DATA_0.input_cmac,
                        USER_DATA_0.output_url, USER_DATA_0.key, "input_data0",
                        "output_data0")

    user1 = DataClient(USER_DATA_1.user_id, USER_DATA_1.password)
    user1.register_data(task_id, USER_DATA_1.input_url, USER_DATA_1.input_cmac,
                        USER_DATA_1.output_url, USER_DATA_1.key, "input_data1",
                        "output_data1")

    user2 = DataClient(USER_DATA_2.user_id, USER_DATA_2.password)
    user2.register_data(task_id, USER_DATA_2.input_url, USER_DATA_2.input_cmac,
                        USER_DATA_2.output_url, USER_DATA_2.key, "input_data2",
                        "output_data2")

    user0.approve_task(task_id)
    user1.approve_task(task_id)
    user2.approve_task(task_id)

    ## USER 3 start the computation
    config_client.run_task(task_id)

    ## USER 0, 1, 2 get the result
    result_user0 = user0.get_task_result(task_id)
    result_user1 = user1.get_task_result(task_id)
    result_user2 = user2.get_task_result(task_id)

    print("[+] User 0 result: " + result_user0.decode("utf-8"))
    print("[+] User 1 result: " + result_user1.decode("utf-8"))
    print("[+] User 2 result: " + result_user2.decode("utf-8"))


if __name__ == '__main__':
    main()
