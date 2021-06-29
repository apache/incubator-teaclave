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

from teaclave import (AuthenticationService, FrontendService,
                      AuthenticationClient, FrontendClient)
from utils import (AUTHENTICATION_SERVICE_ADDRESS, FRONTEND_SERVICE_ADDRESS,
                   AS_ROOT_CA_CERT_PATH, ENCLAVE_INFO_PATH, USER_ID,
                   USER_PASSWORD)


class WASMAddExample:
    def __init__(self, user_id, user_password):
        self.user_id = user_id
        self.user_password = user_password

    def add(self,
            payload_file="wasm_c_simple_add_payload/simple_add.wasm",
            adder1="3",
            adder2="4"):
        client = AuthenticationService(
            AUTHENTICATION_SERVICE_ADDRESS, AS_ROOT_CA_CERT_PATH,
            ENCLAVE_INFO_PATH).connect().get_client()

        print("[+] registering user")
        client.user_register(self.user_id, self.user_password)

        print("[+] login")
        token = client.user_login(self.user_id, self.user_password)

        client = FrontendService(FRONTEND_SERVICE_ADDRESS,
                                 AS_ROOT_CA_CERT_PATH,
                                 ENCLAVE_INFO_PATH).connect().get_client()
        metadata = {"id": self.user_id, "token": token}
        client.metadata = metadata

        print("[+] registering function")

        with open(payload_file, "rb") as f:
            payload = f.read()

        function_id = client.register_function(name="entrypoint",
                                               description="test of wasm",
                                               executor_type="wamr",
                                               payload=list(payload),
                                               arguments=["adder1", "adder2"])

        print("[+] creating task")
        task_id = client.create_task(function_id=function_id,
                                     function_arguments={
                                         "adder1": adder1,
                                         "adder2": adder2
                                     },
                                     executor="wamr")

        print("[+] invoking task")
        client.invoke_task(task_id)

        print("[+] getting result")
        result = client.get_task_result(task_id)
        print("[+] done")

        return bytes(result)


def main():
    example = WASMAddExample(USER_ID, USER_PASSWORD)
    if len(sys.argv) == 2:
        adder1 = sys.argv[1]
        rt = example.add(adder1=adder1)
    elif len(sys.argv) == 3:
        adder1 = sys.argv[1]
        adder2 = sys.argv[2]

        rt = example.add(adder1=adder1, adder2=adder2)
    else:
        rt = example.add()

    print("[+] function return: ", rt)


if __name__ == '__main__':
    main()
