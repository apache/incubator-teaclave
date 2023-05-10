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

from teaclave import FunctionArgument
from utils import connect_authentication_service, connect_frontend_service, PlatformAdmin


class UserData:

    def __init__(self, user_id, password):
        self.user_id = user_id
        self.password = password


USER_DATA_0 = UserData("user0", "password")


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

    def register_function(self, func_name):
        client = self.client

        print(f"[+] {self.user_id} registering function")

        function_id = client.register_function(
            name=func_name,
            description=func_name,
            executor_type="builtin",
            arguments=[FunctionArgument("num_user")],
            inputs=[],
            outputs=[])

        return function_id

    def list_function(self):
        client = self.client

        print(f"[+] {self.user_id} list function")
        functions = client.list_functions(self.user_id)
        return functions

    def disable_function(self, function_id):
        client = self.client

        print(f"[+] {self.user_id} disable function")
        client.disable_function(function_id)


def main():
    platform_admin = PlatformAdmin("admin", "teaclave")
    try:
        platform_admin.register_user(USER_DATA_0.user_id, USER_DATA_0.password)
    except Exception:
        pass

    config_client = ConfigClient(USER_DATA_0.user_id, USER_DATA_0.password)
    function_id1 = config_client.register_function("func_test1")
    function_id2 = config_client.register_function("func_test1")
    functions = config_client.list_function()
    func_nums_before = len(functions['registered_functions'])
    print(f"{func_nums_before} functions registered")
    print(f"Disable {function_id2}")
    config_client.disable_function(function_id2)
    functions = config_client.list_function()
    func_nums_after = len(functions['registered_functions'])
    print(f"{func_nums_after} functions registered")
    assert (func_nums_before == func_nums_after + 1)


if __name__ == '__main__':
    main()
