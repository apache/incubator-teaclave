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


class OptionalFilesTwoParticipants:

    def __init__(self, user_id1, user_password1, user_id2, user_password2):
        self.user_id1 = user_id1
        self.user_password1 = user_password1
        with connect_authentication_service() as client1:
            token1 = client1.user_login(self.user_id1, self.user_password1)

        self.client1 = connect_frontend_service()
        metadata = {"id": self.user_id1, "token": token1}
        self.client1.metadata = metadata

        self.user_id2 = user_id2
        self.user_password2 = user_password2
        with connect_authentication_service() as client2:
            token2 = client2.user_login(self.user_id2, self.user_password2)

        self.client2 = connect_frontend_service()
        metadata = {"id": self.user_id2, "token": token2}
        self.client2.metadata = metadata

    def validate_task(self, task):
        # The task has data from two parties
        if len(task["inputs_ownership"]) != 2:
            return False
        # The data is from user_a and user_b
        if (task["inputs_ownership"][1]['uids'] == ['user_a']
                and task["inputs_ownership"][0]['uids'] == ['user_b']) or (
                    task["inputs_ownership"][1]['uids'] == ['user_b']
                    and task["inputs_ownership"][0]['uids'] == ['user_a']):
            return True
        else:
            return False

    def run_task(self, function_id):
        client1 = self.client1
        client2 = self.client2
        print("[+] registering input file")
        url = "http://localhost:6789/fixtures/functions/gbdt_training/train.enc"
        cmac = [
            0x88, 0x1a, 0xdc, 0xa6, 0xb0, 0x52, 0x44, 0x72, 0xda, 0x0a, 0x9d,
            0x0b, 0xb0, 0x2b, 0x9a, 0xf9
        ]
        schema = "teaclave-file-128"
        key = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        iv = []
        input_file_id1 = client1.register_input_file(url, schema, key, iv,
                                                     cmac)
        input_file_id2 = client2.register_input_file(url, schema, key, iv,
                                                     cmac)

        print("[+] creating task")
        task_id = client1.create_task(function_id=function_id,
                                      function_arguments={},
                                      executor="mesapy",
                                      inputs_ownership=[
                                          OwnerList("input_data1",
                                                    [self.user_id1]),
                                          OwnerList("input_data2",
                                                    [self.user_id2])
                                      ])
        client1.assign_data_to_task(task_id,
                                    [DataMap("input_data1", input_file_id1)],
                                    [])
        client2.assign_data_to_task(task_id,
                                    [DataMap("input_data2", input_file_id2)],
                                    [])

        task1 = client1.get_task(task_id)
        if self.validate_task(task1):
            print("User_a approves the task")
            client1.approve_task(task_id)
        else:
            print("User_a disapproves the task")

        task2 = client2.get_task(task_id)
        if self.validate_task(task2):
            print("User_b approves the task")
            client2.approve_task(task_id)
        else:
            print("User_b disapproves the task")

        client1.invoke_task(task_id)
        result = client1.get_task_result(task_id)
        return bytes(result)


class OptionalFilesOneParticipant:

    def __init__(self, user_id, user_password):
        self.user_id = user_id
        self.user_password = user_password
        with connect_authentication_service() as client:
            print(f"[+] {self.user_id} login")
            token = client.user_login(self.user_id, self.user_password)

        self.client = connect_frontend_service()
        metadata = {"id": self.user_id, "token": token}
        self.client.metadata = metadata

    # The function template defines three optional input files
    def register_function(self):
        print("[+] registering function")
        with open("mesapy_optional_files_payload.py", "rb") as f:
            payload = f.read()
        function_id = self.client.register_function(
            name="mesapy-echo",
            description="An echo function implemented in Python",
            executor_type="python",
            payload=list(payload),
            arguments=[],
            inputs=[
                FunctionInput("input_data1", "Client 0 data.", True),
                FunctionInput("input_data2", "Client 1 data.", True),
                FunctionInput("input_data3", "Client 2 data.", True)
            ])
        return function_id

    def with_input(self, user_id, function_id):
        client = self.client
        print("[+] registering input file")
        url = "http://localhost:6789/fixtures/functions/gbdt_training/train.enc"
        cmac = [
            0x88, 0x1a, 0xdc, 0xa6, 0xb0, 0x52, 0x44, 0x72, 0xda, 0x0a, 0x9d,
            0x0b, 0xb0, 0x2b, 0x9a, 0xf9
        ]
        schema = "teaclave-file-128"
        key = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        iv = []
        input_file_id = client.register_input_file(url, schema, key, iv, cmac)
        print("[+] creating task")
        task_id = client.create_task(
            function_id=function_id,
            function_arguments={},
            executor="mesapy",
            inputs_ownership=[OwnerList("input_data1", [user_id])])
        print("[+] assigning data to task")
        client.assign_data_to_task(task_id,
                                   [DataMap("input_data1", input_file_id)], [])

        print("[+] approving task")
        client.approve_task(task_id)

        print("[+] invoking task")
        client.invoke_task(task_id)
        print("[+] getting result")
        result = client.get_task_result(task_id)
        print("[+] done")
        return bytes(result)

    def without_input(self, user_id, function_id):
        print("[+] creating task")
        client = self.client
        task_id = client.create_task(function_id=function_id,
                                     function_arguments={},
                                     executor="mesapy")

        print("[+] invoking task")
        client.invoke_task(task_id)

        print("[+] getting result")
        result = client.get_task_result(task_id)
        print("[+] done")
        return bytes(result)


def main():

    platform_admin = PlatformAdmin("admin", "teaclave")
    try:
        platform_admin.register_user("user_a", "password")
        platform_admin.register_user("user_b", "password")
    except Exception:
        pass
    task = OptionalFilesOneParticipant("user_a", "password")
    function_id = task.register_function()
    print("Data owners do not register input files")
    rt = task.without_input("user_a", function_id)
    print(rt)
    print("Data owners register input files")
    rt = task.with_input("user_a", function_id)
    print(rt)
    print("The task has more than more participants")
    task = OptionalFilesTwoParticipants("user_a", "password", "user_b",
                                        "password")
    rt = task.run_task(function_id)
    print(rt)


if __name__ == '__main__':
    main()
