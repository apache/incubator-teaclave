#!/usr/bin/env python3

import sys

from teaclave import (AuthenticationService, FrontendService,
                      AuthenticationClient, FrontendClient)
from utils import (AUTHENTICATION_SERVICE_ADDRESS, FRONTEND_SERVICE_ADDRESS,
                   AS_ROOT_CA_CERT_PATH, ENCLAVE_INFO_PATH, USER_ID,
                   USER_PASSWORD)

# In the below example, user 3 creates the task and user 0, 1, 2 upload their private data. 
# Then user 3 invokes the task and user 0, 1, 2 get the result.

USER3_ID = "user3"
USER3_PASSWORD = "password"


USER0_ID = "user0"
USER0_PASSWORD = "password"
USER0_input_url = "http://localhost:6789/fixtures/functions/private_join_and_compute/three_party_data/bank_a.enc"
USER0_output_url = "http://localhost:6789/fixtures/functions/private_join_and_compute/three_party_results/user0_output.enc"
USER0_input_cmac = "7884a62894e7be50b9795ba22ce5ee7f"
USER0_key = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]

USER1_ID = "user1"
USER1_PASSWORD = "password"
USER1_input_url = "http://localhost:6789/fixtures/functions/private_join_and_compute/three_party_data/bank_b.enc"
USER1_output_url = "http://localhost:6789/fixtures/functions/private_join_and_compute/three_party_results/user1_output.enc"
USER1_input_cmac = "75b8e931887bd57564d93df31c282bb9"
USER1_key = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]

USER2_ID = "user2"
USER2_PASSWORD = "password"
USER2_input_url = "http://localhost:6789/fixtures/functions/private_join_and_compute/three_party_data/bank_c.enc"
USER2_output_url = "http://localhost:6789/fixtures/functions/private_join_and_compute/three_party_results/user2_output.enc"
USER2_input_cmac = "35acf29139485067d1ae6212c0577b43"
USER2_key = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]


class FunctionInput:
    def __init__(self, name, description):
        self.name = name
        self.description = description


class FunctionOutput:
    def __init__(self, name, description):
        self.name = name
        self.description = description


class OwnerList:
    def __init__(self, data_name, uids):
        self.data_name = data_name
        self.uids = uids


class DataList:
    def __init__(self, data_name, data_id):
        self.data_name = data_name
        self.data_id = data_id

class ConfigClient:
    def __init__(self, user_id, user_password):
        self.user_id = user_id
        self.user_password = user_password
        self.channel = AuthenticationService(AUTHENTICATION_SERVICE_ADDRESS,
                                        AS_ROOT_CA_CERT_PATH,
                                        ENCLAVE_INFO_PATH).connect()
        self.client = AuthenticationClient(self.channel)
        print("[+] registering user")
        self.client.user_register(self.user_id, self.user_password)
        print("[+] login")
        token = self.client.user_login(self.user_id, self.user_password)
        self.channel = FrontendService(FRONTEND_SERVICE_ADDRESS,
                                  AS_ROOT_CA_CERT_PATH,
                                  ENCLAVE_INFO_PATH).connect()
        metadata = {"id": self.user_id, "token": token}
        self.client = FrontendClient(self.channel, metadata)

    def set_task(self):
        client = self.client;
        channel = self.channel;
 

        print("[+] registering function")

        function_id = client.register_function(
            name="builtin-private-join-and-compute",
            description="Native Private Join And Compute",
            executor_type="builtin",
            arguments=[
                "num_user"
            ],
            inputs=[
                FunctionInput("input_data0", "Bank A data file."),
                FunctionInput("input_data1", "Bank B data file."),
                FunctionInput("input_data2", "Bank C data file.")
            ],
            outputs=[FunctionOutput("output_data0", "Output data."),
                     FunctionOutput("output_data1", "Output data."),
                     FunctionOutput("output_data2", "Output date.")
                    ])

        print("[+] creating task")
        task_id = client.create_task(
            function_id=function_id,
            function_arguments=({
                "num_user": 3,
            }),
            executor="builtin",
            inputs_ownership=[OwnerList("input_data0", [USER0_ID]), OwnerList("input_data1", [USER1_ID]), OwnerList("input_data2", [USER2_ID])],
            outputs_ownership=[OwnerList("output_data0", [USER0_ID]), OwnerList("output_data1", [USER1_ID]), OwnerList("output_data2", [USER2_ID])])
        
        return task_id

    def run_task(self, task_id):
        client = self.client;
        channel = self.channel;
        
        client.approve_task(task_id)
        print("[+] invoking task")
        client.invoke_task(task_id)

        print("[+] getting result")
        result = client.get_task_result(task_id)
        print("[+] done")

        return bytes(result)

class DataClient:
    def __init__(self, user_id, user_password):
        self.user_id = user_id
        self.user_password = user_password
        self.channel = AuthenticationService(AUTHENTICATION_SERVICE_ADDRESS,
                                        AS_ROOT_CA_CERT_PATH,
                                        ENCLAVE_INFO_PATH).connect()
        self.client = AuthenticationClient(self.channel)
        print("[+] registering user")
        self.client.user_register(self.user_id, self.user_password)
        print("[+] login")
        token = self.client.user_login(self.user_id, self.user_password)
        self.channel = FrontendService(FRONTEND_SERVICE_ADDRESS,
                                  AS_ROOT_CA_CERT_PATH,
                                  ENCLAVE_INFO_PATH).connect()
        metadata = {"id": self.user_id, "token": token}
        self.client = FrontendClient(self.channel, metadata)
    
    def register_data(self, task_id, input_url, input_cmac, output_url, file_key, input_label, output_label):
        client = self.client;
        channel = self.channel;

        print("[+] registering input file")
        url = input_url
        cmac = input_cmac
        schema = "teaclave-file-128"
        key = file_key
        iv = []
        training_data_id = client.register_input_file(url, schema, key, iv,
                                                      cmac)
        print("[+] registering output file")
        url = output_url
        schema = "teaclave-file-128"
        key = file_key
        iv = []
        output_model_id = client.register_output_file(url, schema, key, iv)

        print("[+] assigning data to task")
        client.assign_data_to_task(
            task_id, 
            [DataList(input_label, training_data_id)],
            [DataList(output_label, output_model_id)])

        print("[+] data client approving task")
 
    def approve_task(self, task_id):
        client = self.client;
        channel = self.channel;
        client.approve_task(task_id)


def main():

    config_client = ConfigClient(USER3_ID, USER3_PASSWORD)
    task_id = config_client.set_task()

    user0 = DataClient(USER0_ID, USER0_PASSWORD)
    user0.register_data(task_id, USER0_input_url, USER0_input_cmac, USER0_output_url, USER0_key, "input_data0", "output_data0")

    user1 = DataClient(USER1_ID, USER1_PASSWORD)
    user1.register_data(task_id, USER1_input_url, USER1_input_cmac, USER1_output_url, USER1_key, "input_data1", "output_data1")

    user2 = DataClient(USER2_ID, USER2_PASSWORD)
    user2.register_data(task_id, USER2_input_url, USER2_input_cmac, USER2_output_url, USER2_key, "input_data2", "output_data2")

    user0.approve_task(task_id)
    user1.approve_task(task_id)
    user2.approve_task(task_id)

    rt = config_client.run_task(task_id)
    print("[+] function return: ", rt)

if __name__ == '__main__':
    main()
