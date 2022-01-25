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
import time

from teaclave import FunctionInput, FunctionOutput, OwnerList, DataMap
from utils import USER_ID, USER_PASSWORD, connect_authentication_service, connect_frontend_service, PlatformAdmin


class MesaPyEchoExample:

    def __init__(self, user_id, user_password):
        self.user_id = user_id
        self.user_password = user_password

    def deadloop(self, payload_file="mesapy_deadloop_payload.py"):
        with connect_authentication_service() as client:
            print(f"[+] {self.user_id} login")
            token = client.user_login(self.user_id, self.user_password)

        client = connect_frontend_service()
        metadata = {"id": self.user_id, "token": token}
        client.metadata = metadata

        print("[+] registering function")

        with open(payload_file, "rb") as f:
            payload = f.read()
        function_id = client.register_function(
            name="mesapy-deadloop",
            description="A deadloop function to test task cancellation",
            executor_type="python",
            payload=list(payload),
            arguments=[])

        print("[+] creating task")
        task_id = client.create_task(function_id=function_id,
                                     function_arguments={},
                                     executor="mesapy")

        print("[+] invoking task")
        client.invoke_task(task_id)

        print("[+] canceling task")
        time.sleep(5)
        client.cancel_task(task_id)

        print("[+] getting result")

        try:
            result = client.get_task_result(task_id)
        except Exception as e:
            print(f"[+] result: {str(e)}")
            result = str(e)

        return result


def main():
    example = MesaPyEchoExample(USER_ID, USER_PASSWORD)
    rv = example.deadloop()

    print("[+] function return: ", rv)


if __name__ == '__main__':
    main()
