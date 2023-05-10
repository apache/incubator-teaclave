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

# In the example, user 0 creates the task and user 0, 1, upload their private data.
# Then user 0 invokes the task and user 0, 1 get the result.


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


# If you're using `docker-compose` to start the Teaclave server containers,
# please change `localhost` to `teaclave-file-service`
INPUT_FILE_URL_PREFIX = "http://localhost:6789/fixtures/functions/ordered_set_intersect/"
OUTPUT_FILE_URL_PREFIX = "http://localhost:6789/fixtures/functions/ordered_set_intersect/"

USER_DATA_0 = UserData("user0", "password",
                       INPUT_FILE_URL_PREFIX + "psi0.txt.enc",
                       OUTPUT_FILE_URL_PREFIX + "output_psi0.enc", [
                           0x92, 0xf6, 0x86, 0xd4, 0xac, 0x2b, 0xa6, 0xb4,
                           0xff, 0xd9, 0x3b, 0xc7, 0xac, 0x5d, 0xbf, 0x58
                       ], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])

USER_DATA_1 = UserData("user1", "password",
                       INPUT_FILE_URL_PREFIX + "psi1.txt.enc",
                       OUTPUT_FILE_URL_PREFIX + "output_psi1.enc", [
                           0x8b, 0x31, 0x04, 0x97, 0x2a, 0x6f, 0x0d, 0xe9,
                           0x49, 0x31, 0x5e, 0x0b, 0x45, 0xd5, 0xdd, 0x66
                       ], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])


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
            name="builtin-ordered-set-intersect",
            description="Native Private Set Intersection",
            executor_type="builtin",
            arguments=[FunctionArgument("order")],
            inputs=[
                FunctionInput("input_data1", "Client 0 data."),
                FunctionInput("input_data2", "Client 1 data.")
            ],
            outputs=[
                FunctionOutput("output_result1", "Output data."),
                FunctionOutput("output_result2", "Output data.")
            ])

        print(f"[+] {self.user_id} creating task")
        task_id = client.create_task(
            function_id=function_id,
            function_arguments=({
                "order": "ascending",  # Order can be ascending or desending 
            }),
            executor="builtin",
            inputs_ownership=[
                OwnerList("input_data1", [USER_DATA_0.user_id]),
                OwnerList("input_data2", [USER_DATA_1.user_id])
            ],
            outputs_ownership=[
                OwnerList("output_result1", [USER_DATA_0.user_id]),
                OwnerList("output_result2", [USER_DATA_1.user_id])
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
    # Register user
    platform_admin = PlatformAdmin("admin", "teaclave")
    try:
        platform_admin.register_user(USER_DATA_0.user_id, USER_DATA_0.password)
        platform_admin.register_user(USER_DATA_1.user_id, USER_DATA_1.password)
    except Exception:
        pass

    user0 = Client(USER_DATA_0.user_id, USER_DATA_0.password)
    user1 = Client(USER_DATA_1.user_id, USER_DATA_1.password)

    task_id = user0.set_task()

    user0.register_data(task_id, USER_DATA_0.input_url, USER_DATA_0.input_cmac,
                        USER_DATA_0.output_url, USER_DATA_0.key, "input_data1",
                        "output_result1")

    user1.register_data(task_id, USER_DATA_1.input_url, USER_DATA_1.input_cmac,
                        USER_DATA_1.output_url, USER_DATA_1.key, "input_data2",
                        "output_result2")

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
