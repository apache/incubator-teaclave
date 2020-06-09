#!/usr/bin/env python3

import sys

from teaclave import (AuthenticationService, FrontendService,
                      AuthenticationClient, FrontendClient, FunctionInput,
                      FunctionOutput, OwnerList, DataMap)
from utils import (AUTHENTICATION_SERVICE_ADDRESS, FRONTEND_SERVICE_ADDRESS,
                   AS_ROOT_CA_CERT_PATH, ENCLAVE_INFO_PATH, USER_ID,
                   USER_PASSWORD)

# In the example, user 3 creates the task and user 0, 1, 2 upload their private data.
# Then user 3 invokes the task and user 0, 1, 2 get the result.


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


class DataList:
    def __init__(self, data_name, data_id):
        self.data_name = data_name
        self.data_id = data_id


class ConfigClient:
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
            name="builtin-private-join-and-compute",
            description="Native Private Join And Compute",
            executor_type="builtin",
            arguments=["num_user"],
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
                                                   [user0_data.user_id]),
                                         OwnerList("input_data1",
                                                   [user1_data.user_id]),
                                         OwnerList("input_data2",
                                                   [user2_data.user_id])
                                     ],
                                     outputs_ownership=[
                                         OwnerList("output_data0",
                                                   [user0_data.user_id]),
                                         OwnerList("output_data1",
                                                   [user1_data.user_id]),
                                         OwnerList("output_data2",
                                                   [user2_data.user_id])
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

    def register_data(self, task_id, input_url, input_cmac, output_url,
                      file_key, input_label, output_label):
        client = self.client

        print(f"[+] {self.user_id} registering input file")
        url = input_url
        cmac = input_cmac
        schema = "teaclave-file-128"
        key = file_key
        iv = []
        training_data_id = client.register_input_file(url, schema, key, iv,
                                                      cmac)
        print(f"[+] {self.user_id} registering output file")
        url = output_url
        schema = "teaclave-file-128"
        key = file_key
        iv = []
        output_model_id = client.register_output_file(url, schema, key, iv)

        print(f"[+] {self.user_id} assigning data to task")
        client.assign_data_to_task(task_id,
                                   [DataList(input_label, training_data_id)],
                                   [DataList(output_label, output_model_id)])

    def approve_task(self, task_id):
        client = self.client
        print(f"[+] {self.user_id} approving task")
        client.approve_task(task_id)

    def get_task_result(self, task_id):
        client = self.client
        print(f"[+] {self.user_id} getting task result")
        return bytes(client.get_task_result(task_id))


def main():

    user0_data = UserData(
        "user0", "password",
        "http://localhost:6789/fixtures/functions/private_join_and_compute/three_party_data/bank_a.enc",
        "http://localhost:6789/fixtures/functions/private_join_and_compute/three_party_results/user0_output.enc",
        "7884a62894e7be50b9795ba22ce5ee7f",
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])

    user1_data = UserData(
        "user1", "password",
        "http://localhost:6789/fixtures/functions/private_join_and_compute/three_party_data/bank_b.enc",
        "http://localhost:6789/fixtures/functions/private_join_and_compute/three_party_results/user1_output.enc",
        "75b8e931887bd57564d93df31c282bb9",
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])

    user2_data = UserData(
        "user2", "password",
        "http://localhost:6789/fixtures/functions/private_join_and_compute/three_party_data/bank_c.enc",
        "http://localhost:6789/fixtures/functions/private_join_and_compute/three_party_results/user2_output.enc",
        "35acf29139485067d1ae6212c0577b43",
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])

    user3_data = UserData("user3", "password")

    ## USER 3 creates the task
    config_client = ConfigClient(user3_data.user_id, user3_data.password)
    task_id = config_client.set_task()

    ## USER 0, 1, 2 join the task and upload their data
    user0 = DataClient(user0_data.user_id, user0_data.password)
    user0.register_data(task_id, user0_data.input_url, user0_data.input_cmac,
                        user0_data.output_url, user0_data.key, "input_data0",
                        "output_data0")

    user1 = DataClient(user1_data.user_id, user1_data.password)
    user1.register_data(task_id, user1_data.input_url, user1_data.input_cmac,
                        user1_data.output_url, user1_data.key, "input_data1",
                        "output_data1")

    user2 = DataClient(user2_data.user_id, user2_data.password)
    user2.register_data(task_id, user2_data.input_url, user2_data.input_cmac,
                        user2_data.output_url, user2_data.key, "input_data2",
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
