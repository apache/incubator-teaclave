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
"""
An example about Logistic Regression in MesaPy.
"""

import sys
import binascii
from typing import List
from teaclave import (AuthenticationService, FrontendService,
                      AuthenticationClient, FrontendClient, FunctionInput,
                      FunctionOutput, OwnerList, DataMap)
from utils import (AUTHENTICATION_SERVICE_ADDRESS, FRONTEND_SERVICE_ADDRESS,
                   AS_ROOT_CA_CERT_PATH, ENCLAVE_INFO_PATH, USER_ID,
                   USER_PASSWORD)

from enum import Enum


class Executor(Enum):
    builtin = "builtin"
    python = "mesapy"


class UserClient:
    def __init__(self, user_id, password):
        self.user_id = user_id
        self.password = password


class InputData:
    def __init__(self,
                 input_url="",
                 input_cmac="",
                 key=[],
                 label="input_data_0",
                 schema="teaclave-file-128",
                 iv=[]):
        self.input_url = input_url
        self.input_cmac = input_cmac
        self.file_key = key
        self.label = label
        self.schema = schema
        self.iv = iv

    def set_label(self, label="input_data_0"):
        self.label = label


class OutputData:
    def __init__(self,
                 output_url="",
                 key=[],
                 label="result_data_0",
                 schema="teaclave-file-128",
                 iv=[]):
        self.output_url = output_url
        self.file_key = key
        self.schema = schema
        self.iv = iv
        self.label = label

    def set_label(self, label="result_data_0"):
        self.label = label


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

    def set_single_party_task(self,
                              functionname,
                              payloadpath,
                              args={},
                              inlabels=["input_data_0"],
                              outlabels=["result_data_0"],
                              ex=Executor.builtin):
        client = self.client
        print(f"[+] {self.user_id} registering function")
        p_str = ""
        if payloadpath != "":
            print(f"[+] {self.user_id} reading payload file")
            with open(payloadpath, "rb") as f:
                p_str = f.read()

        function_id = client.register_function(
            name=functionname,
            description="worker: %s" % functionname,
            executor_type=ex.name,
            arguments=list(args.keys()),
            payload=list(p_str),
            inputs=[
                FunctionInput(label, "user input data file： %s" % label)
                for label in inlabels
            ],
            outputs=[
                FunctionOutput(label, "user output file: %s" % label)
                for label in outlabels
            ])

        print(f"[+] {self.user_id} creating task")
        task_id = client.create_task(function_id=function_id,
                                     executor=ex.value,
                                     function_arguments=(args),
                                     inputs_ownership=[
                                         OwnerList(label, [self.user_id])
                                         for label in inlabels
                                     ],
                                     outputs_ownership=[
                                         OwnerList(label, [self.user_id])
                                         for label in outlabels
                                     ])

        return task_id

    def run_task(self, task_id):
        client = self.client
        client.approve_task(task_id)
        print(f"[+] {self.user_id} invoking task")
        client.invoke_task(task_id)

    def register_data(self, task_id, inputs: List[InputData],
                      outputs: List[OutputData]):
        client = self.client
        print(f"[+] {self.user_id} registering input file")
        input_data_list = []
        for da in inputs:
            url = da.input_url
            cmac = da.input_cmac
            schema = da.schema
            key = da.file_key
            iv = da.iv
            input_id = client.register_input_file(url, schema, key, iv, cmac)
            input_data_list.append(DataList(da.label, input_id))
        print(f"[+] {self.user_id} registering output file")
        output_data_list = []
        for out_data in outputs:
            out_url = out_data.output_url
            schema = out_data.schema
            key = out_data.file_key
            iv = out_data.iv
            output_id = client.register_output_file(out_url, schema, key, iv)
            output_data_list.append(DataList(out_data.label, output_id))
        print(f"[+] {self.user_id} assigning data to task")

        client.assign_data_to_task(task_id, input_data_list, output_data_list)
        return True

    def approve_task(self, task_id):
        client = self.client
        print(f"[+] {self.user_id} approving task")
        client.approve_task(task_id)

    def get_task_result(self, task_id):
        client = self.client
        print(f"[+] {self.user_id} getting task result")
        return bytes(client.get_task_result(task_id))

    def get_output_cmac_by_tag(self, task_id, tag):
        client = self.client
        print(f"[+] {self.user_id} getting task output")
        return client.get_output_cmac_by_tag(task_id, tag)


train = "http://localhost:6789/fixtures/functions/py_logistic_reg/train.enc"
predict = "http://localhost:6789/fixtures/functions/py_logistic_reg/predict.enc"
params = "http://localhost:6789/fixtures/functions/py_logistic_reg/params.out"
scaler = "http://localhost:6789/fixtures/functions/py_logistic_reg/scaler.out"
USER = UserClient("user0", "password")
fo_test = "http://localhost:6789/fixtures/functions/py_logistic_reg/testa.out"

train_inputs = [
    InputData(train,
              "057baff3598121d05cec6c5e9ce2aa39",
              [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
              label="input_train")
]
train_outputs = [
    OutputData(params, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
               label="output_params"),
    OutputData(scaler, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
               label="output_scaler")
]
out_tests = OutputData(fo_test,
                       [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                       label="out_tests")


def main():
    print("[+] mesapy_logistic_reg_train_task begin!")
    tc = ConfigClient(USER.user_id, USER.password)
    train_task_id = tc.set_single_party_task(
        "mesapy_logistic_reg_train_task",
        "./mesapy_logistic_reg_payload.py", {
            "train_file": "input_train",
            "operation": "train",
            "params_saved": "output_params",
            "scaler_saved": "output_scaler"
        }, ["input_train"], ["output_params", "output_scaler"],
        ex=Executor.python)
    tc.register_data(train_task_id, train_inputs, train_outputs)
    tc.run_task(train_task_id)
    train_task_result = tc.get_task_result(train_task_id)
    print("[+] User 0 result: " + train_task_result.decode("utf-8"))
    print("[+] mesapy_logistic_reg_predict_task begin!")
    predict_task_id = tc.set_single_party_task(
        "mesapy_logistic_reg_predict_task",
        "./mesapy_logistic_reg_payload.py", {
            "operation": "predict",
            "predict_file": "input_predict",
            "params_saved": "output_params",
            "scaler_saved": "output_scaler"
        }, ["input_predict", "output_params", "output_scaler"], [],
        ex=Executor.python)
    output_params_cmac = tc.get_output_cmac_by_tag(train_task_id,
                                                   "output_params")
    output_scaler_cmac = tc.get_output_cmac_by_tag(train_task_id,
                                                   "output_scaler")
    predict_inputs = [
        InputData(predict,
                  "ae168afac768fd53e5d90d8e4b594d27",
                  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                  label="input_predict"),
        InputData(params,
                  output_params_cmac,
                  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                  label="output_params"),
        InputData(scaler,
                  output_scaler_cmac,
                  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                  label="output_scaler")
    ]

    tc.register_data(predict_task_id, predict_inputs, [])
    tc.run_task(predict_task_id)
    predict_task_result = tc.get_task_result(predict_task_id)
    print("[+] Predict result: " + predict_task_result.decode("utf-8"))
    print("[+] logistic_reg_task end!")


if __name__ == '__main__':
    main()
