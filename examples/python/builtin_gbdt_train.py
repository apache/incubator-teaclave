#!/usr/bin/env python3

import sys

from teaclave import (
    AuthenticationService,
    FrontendService,
    AuthenticationClient,
    FrontendClient
)
from utils import (
    AUTHENTICATION_SERVICE_ADDRESS,
    FRONTEND_SERVICE_ADDRESS,
    AS_ROOT_CA_CERT_PATH,
    ENCLAVE_INFO_PATH,
    USER_ID,
    USER_PASSWORD
)

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

class BuiltinGbdtExample:
    def __init__(self, user_id, user_password):
        self.user_id = user_id
        self.user_password = user_password

    def gbdt(self, message="Hello, Teaclave!"):
        channel = AuthenticationService(AUTHENTICATION_SERVICE_ADDRESS,
                                        AS_ROOT_CA_CERT_PATH,
                                        ENCLAVE_INFO_PATH).connect()
        client = AuthenticationClient(channel)

        print("[+] registering user")
        client.user_register(self.user_id, self.user_password)

        print("[+] login")
        token = client.user_login(self.user_id, self.user_password)

        channel = FrontendService(FRONTEND_SERVICE_ADDRESS,
                                  AS_ROOT_CA_CERT_PATH,
                                  ENCLAVE_INFO_PATH).connect()
        metadata = {"id": self.user_id, "token": token}
        client = FrontendClient(channel, metadata)

        print("[+] registering function")
        function_id = client.register_function(
            name="builtin-gbdt-train",
            description="Native Gbdt Training Function",
            executor_type="builtin",
            arguments=[
                "feature_size",
                "max_depth",
                "iterations",
                "shrinkage",
                "feature_sample_ratio",
                "data_sample_ratio",
                "min_leaf_size",
                "loss",
                "training_optimization_level"],
            inputs=[FunctionInput("training_data", "Input traning data file.")],
            outputs=[FunctionOutput("trained_model", "Output trained model.")])

        print("[+] registering input file")
        url = "http://localhost:6789/fixtures/functions/gbdt_training/train.enc"
        cmac = "881adca6b0524472da0a9d0bb02b9af9"
        schema = "teaclave-file-128"
        key = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        iv = []
        training_data_id = client.register_input_file(url, schema, key, iv, cmac)

        print("[+] registering output file")
        url = "http://localhost:6789/fixtures/functions/gbdt_training/e2e_output_model.enc"
        schema = "teaclave-file-128"
        key = [63, 195, 250, 208, 252, 127, 203, 27, 247, 168, 71, 77, 27, 47, 254, 240]
        iv = []
        output_model_id = client.register_output_file(url, schema, key, iv)

        print("[+] creating task")
        task_id = client.create_task(function_id=function_id,
                                     function_arguments=({
                                         "feature_size": 4,
                                         "max_depth": 4,
                                         "iterations": 100,
                                         "shrinkage": 0.1,
                                         "feature_sample_ratio": 1.0,
                                         "data_sample_ratio": 1.0,
                                         "min_leaf_size": 1,
                                         "loss": "LAD",
                                         "training_optimization_level": 2}),
                                     executor="builtin",
                                     inputs_ownership=[OwnerList("training_data", [self.user_id])],
                                     outputs_ownership=[OwnerList("trained_model", [self.user_id])])

        print("[+] assigning data to task")
        client.assign_data_to_task(task_id, [DataList("training_data", training_data_id)], [DataList("trained_model", output_model_id)])

        print("[+] approving task")
        client.approve_task(task_id)

        print("[+] invoking task")
        client.invoke_task(task_id)

        print("[+] getting result")
        result = client.get_task_result(task_id)
        print("[+] done")

        return bytes(result)


def main():
    example = BuiltinGbdtExample(USER_ID, USER_PASSWORD)
    if len(sys.argv) > 1:
        message = sys.argv[1]
        rt = example.gbdt(message)
    else:
        rt = example.gbdt()

    print("[+] function return: ", rt)


if __name__ == '__main__':
    main()
