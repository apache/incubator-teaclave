#!/usr/bin/env python3

import sys

from teaclave import (AuthenticationService, FrontendService,
                      AuthenticationClient, FrontendClient, FunctionInput,
                      FunctionOutput, OwnerList, DataMap)
from utils import (AUTHENTICATION_SERVICE_ADDRESS, FRONTEND_SERVICE_ADDRESS,
                   AS_ROOT_CA_CERT_PATH, ENCLAVE_INFO_PATH, USER_ID,
                   USER_PASSWORD)

# In the example, user 0 creates the task and user 0, 1, upload their private data.
# Then user 0 invokes the task and user 0, 1 get the result.


class UserData:
    def __init__(self,
                 user_id,
                 password,
                 input_url="",
                 output_url="",
                 input_cmac="",
                 key=[]):
        self.user_id = user_id
        self.password = password
        self.input_url = input_url
        self.output_url = output_url
        self.input_cmac = input_cmac
        self.key = key


INPUT_FILE_URL_PREFIX = "http://localhost:6789/fixtures/functions/psi/"
OUTPUT_FILE_URL_PREFIX = "http://localhost:6789/fixtures/functions/psi/"

USER_DATA_0 = UserData("user0", "password",
                       INPUT_FILE_URL_PREFIX + "psi0.txt.enc",
                       OUTPUT_FILE_URL_PREFIX + "output_psi0.enc",
                       "c0cd36fbd9efdb44e3b4e1a50094f28e",
                       [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])

USER_DATA_1 = UserData("user1", "password",
                       INPUT_FILE_URL_PREFIX + "psi1.txt.enc",
                       OUTPUT_FILE_URL_PREFIX + "output_psi1.enc",
                       "7ebefe1566729bc8d2299f8572ccb458",
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

    def set_task(self):
        client = self.client

        print(f"[+] {self.user_id} registering function")

        function_id = client.register_function(
            name="builtin-private-set-intersection",
            description="Native Private Set Intersection",
            executor_type="builtin",
            inputs=[
                FunctionInput("input_data1", "Client 0 data."),
                FunctionInput("input_data2", "Client 1 data.")
            ],
            outputs=[
                FunctionOutput("output_result1", "Output data."),
                FunctionOutput("output_result2", "Output date.")
            ])

        print(f"[+] {self.user_id} creating task")
        task_id = client.create_task(function_id=function_id,
                                     function_arguments=({}),
                                     executor="builtin",
                                     inputs_ownership=[
                                         OwnerList("input_data1",
                                                   [USER_DATA_0.user_id]),
                                         OwnerList("input_data2",
                                                   [USER_DATA_1.user_id])
                                     ],
                                     outputs_ownership=[
                                         OwnerList("output_result1",
                                                   [USER_DATA_0.user_id]),
                                         OwnerList("output_result2",
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
                        USER_DATA_0.output_url, USER_DATA_0.key, "input_data1",
                        "output_result1")

    user1.register_data(task_id, USER_DATA_1.input_url, USER_DATA_1.input_cmac,
                        USER_DATA_1.output_url, USER_DATA_1.key, "input_data2",
                        "output_result2")

    user0.approve_task(task_id)
    user1.approve_task(task_id)

    ## USER 0 start the computation
    user0.run_task(task_id)

    ## USER 0, 1, 2 get the result
    result_user0 = user0.get_task_result(task_id)
    result_user1 = user1.get_task_result(task_id)

    print("[+] User 0 result: " + result_user0.decode("utf-8"))
    print("[+] User 1 result: " + result_user1.decode("utf-8"))


if __name__ == '__main__':
    main()
