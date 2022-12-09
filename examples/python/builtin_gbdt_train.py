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
from utils import USER_ID, USER_PASSWORD, connect_authentication_service, connect_frontend_service


class BuiltinGbdtExample:

    def __init__(self, user_id, user_password):
        self.user_id = user_id
        self.user_password = user_password

    def gbdt(self):
        with connect_authentication_service() as client:
            print("[+] login")
            token = client.user_login(self.user_id, self.user_password)

        client = connect_frontend_service()
        metadata = {"id": self.user_id, "token": token}
        client.metadata = metadata

        print("[+] registering function")
        function_id = client.register_function(
            name="builtin-gbdt-train",
            description="Native Gbdt Training Function",
            executor_type="builtin",
            arguments=[
                "feature_size", "max_depth", "iterations", "shrinkage",
                "feature_sample_ratio", "data_sample_ratio", "min_leaf_size",
                "loss", "training_optimization_level"
            ],
            inputs=[
                FunctionInput("training_data", "Input traning data file.")
            ],
            outputs=[FunctionOutput("trained_model", "Output trained model.")])

        print("[+] registering input file")
        url = "http://localhost:6789/fixtures/functions/gbdt_training/train.enc"
        cmac = [
            0x86, 0x00, 0x30, 0x49, 0x59, 0x09, 0xb8, 0x48, 0x64, 0xb9, 0x91,
            0x86, 0x5e, 0x9a, 0xd9, 0x4f
        ]
        schema = "teaclave-file-128"
        key = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        iv = []
        training_data_id = client.register_input_file(url, schema, key, iv,
                                                      cmac)

        print("[+] registering output file")
        url = "http://localhost:6789/fixtures/functions/gbdt_training/e2e_output_model.enc"
        schema = "teaclave-file-128"
        key = [
            63, 195, 250, 208, 252, 127, 203, 27, 247, 168, 71, 77, 27, 47,
            254, 240
        ]
        iv = []
        output_model_id = client.register_output_file(url, schema, key, iv)

        print("[+] creating task")
        task_id = client.create_task(
            function_id=function_id,
            function_arguments=({
                "feature_size": 4,
                "max_depth": 4,
                "iterations": 100,
                "shrinkage": 0.1,
                "feature_sample_ratio": 1.0,
                "data_sample_ratio": 1.0,
                "min_leaf_size": 1,
                "loss": "LAD",
                "training_optimization_level": 2
            }),
            executor="builtin",
            inputs_ownership=[OwnerList("training_data", [self.user_id])],
            outputs_ownership=[OwnerList("trained_model", [self.user_id])])

        print("[+] assigning data to task")
        client.assign_data_to_task(
            task_id, [DataMap("training_data", training_data_id)],
            [DataMap("trained_model", output_model_id)])

        print("[+] approving task")
        client.approve_task(task_id)

        print("[+] invoking task")
        client.invoke_task(task_id)

        # If you feel like you don't want to wait for the task to finish
        # or something unexpected happens,
        # you may uncomment the following lines to cancel the task
        # time.sleep(3)
        # print("[+] canceling task")
        # client.cancel_task(task_id)

        print("[+] getting result")
        result = client.get_task_result(task_id)
        print("[+] done")
        client.close()

        return bytes(result)


def main():
    example = BuiltinGbdtExample(USER_ID, USER_PASSWORD)
    rt = example.gbdt()

    print("[+] function return: ", rt)


if __name__ == '__main__':
    main()
