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
Python package `teaclave` is the client SDK for Python developers, providing
some essential data structures, service, and client classes to establish
trusted TLS channel and communicate with Teaclave services (e.g., the
authentication service and frontend service) through RPC protocols.
"""

import struct
import json
import base64
import toml
import os
import time
import ssl
import socket

from typing import Tuple, Dict, List, Any

from cryptography import x509
from cryptography.hazmat.backends import default_backend

from OpenSSL.crypto import load_certificate, FILETYPE_PEM, FILETYPE_ASN1
from OpenSSL.crypto import X509Store, X509StoreContext
from OpenSSL import crypto

__all__ = [
    'FrontendClient', 'FrontendService', 'AuthenticationClient',
    'AuthenticationService', 'FunctionInput', 'FunctionOutput', 'OwnerList',
    'DataMap'
]

Metadata = Dict[str, str]


class FunctionInput:
    """Function input for registering.

    Args:
        name: Name of input data.
        description: Description of the input data.
    """
    def __init__(self, name: str, description: str):
        self.name = name
        self.description = description


class FunctionOutput:
    """Function output for registering.

    Args:
        name: Name of output data.
        description: Description of the output data.
    """
    def __init__(self, name: str, description: str):
        self.name = name
        self.description = description


class OwnerList:
    """Defines data ownership.

    Args:
        data_name: Name of output data.
        uids: A list of user id which own this data.
    """
    def __init__(self, data_name: str, uids: List[str]):
        self.data_name = data_name
        self.uids = uids


class DataMap:
    """Assign data id to input or output data.

    Args:
        data_name: Name of output data.
        data_id: Id for the data name.
    """
    def __init__(self, data_name, data_id):
        self.data_name = data_name
        self.data_id = data_id


class CryptoInfo:
    """Cryptographic information for the input/output data.

    Args:
        schema: Encryption algorithms for the input/output data.
        key: Key for encryption and decryption, bytes in list.
        iv: IV, bytes in list.
    """
    def __init__(self, schema: str, key: List[int], iv: List[int]):
        self.schema = schema
        self.key = key
        self.iv = iv


class UserRegisterReqeust:
    def __init__(self, user_id: str, user_password: str):
        self.request = "user_register"
        self.id = user_id
        self.password = user_password


class UserLoginRequest:
    def __init__(self, user_id: str, user_password: str):
        self.request = "user_login"
        self.id = user_id
        self.password = user_password


class AuthenticationService:
    """
    Establish trusted channel with the authentication service and provide
    clients to send request through RPC.

    Args:
        address: The address of the remote services in tuple.
        as_root_ca_cert_path: Root CA certification of the attestation services
            to verify the attestation report.
        enclave_info_path: Path of enclave info to verify the remote service in
            the attestation report.
    """
    _context = ssl._create_unverified_context()
    _channel = None

    def __init__(self, address: Tuple[str, int], as_root_ca_cert_path: str,
                 enclave_info_path: str):
        self.address = address
        self.as_root_ca_cert_path = as_root_ca_cert_path
        self.enclave_info_path = enclave_info_path

    def connect(self):
        """Establish trusted connection and verify remote attestation report.

        Returns:
            AuthenticationService: The original object which can be chained
                with other methods.
        """
        sock = socket.create_connection(self.address)
        channel = self._context.wrap_socket(sock,
                                            server_hostname=self.address[0])
        cert = channel.getpeercert(binary_form=True)
        _verify_report(self.as_root_ca_cert_path, self.enclave_info_path, cert,
                       "authentication")

        self._channel = channel

        return self

    def get_client(self):
        """Get a client of authentication service to send RPC requests.

        Returns:
            AuthenticationClient: Used for send/receive RPC requests.
        """
        return AuthenticationClient(self._channel)


class AuthenticationClient:
    """Client to communicate with the authentication service.

    Args:
        channel: Trusted TLS socket (verified with remote attestation).
    """
    def __init__(self, channel: ssl.SSLSocket):
        self.channel = channel

    def user_register(self, user_id: str, user_password: str):
        """Register a new user.

        Args:
            user_id: User ID.
            user_password: Password.
        """
        request = UserRegisterReqeust(user_id, user_password)
        _write_message(self.channel, request)
        _ = _read_message(self.channel)

    def user_login(self, user_id: str, user_password: str) -> str:
        """Login and get a session token.

        Args:
            user_id: User ID.
            user_password: Password.

        Returns:
            str: User login token.
        """
        request = UserLoginRequest(user_id, user_password)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        return response["content"]["token"]


class FrontendService:
    """Establish trusted channel with the frontend service and provide
    clients to send request through RPC.

    Args:
        address: The address of the remote services in tuple.
        as_root_ca_cert_path: Root CA certification of the attestation services
            to verify the attestation report.
        enclave_info_path: Path of enclave info to verify the remote service in
            the attestation report.
    """
    _context = ssl._create_unverified_context()
    _channel = None

    def __init__(self, address: Tuple[str, int], as_root_ca_cert_path: str,
                 enclave_info_path: str):
        self.address = address
        self.as_root_ca_cert_path = as_root_ca_cert_path
        self.enclave_info_path = enclave_info_path

    def connect(self):
        """Establish trusted connection and verify remote attestation report.

        Returns:
            FrontendService: The original object which can be chained
                with other methods.
        """
        sock = socket.create_connection(self.address)
        channel = self._context.wrap_socket(sock,
                                            server_hostname=self.address[0])
        cert = channel.getpeercert(binary_form=True)
        _verify_report(self.as_root_ca_cert_path, self.enclave_info_path, cert,
                       "frontend")

        self._channel = channel
        return self

    def get_client(self):
        """Get a client of frontend service to send RPC requests.

        Returns:
            FrontendClient: Used for send/receive RPC requests.
        """
        return FrontendClient(self._channel)


class RegisterFunctionRequest:
    def __init__(self, metadata: Metadata, name: str, description: str,
                 executor_type: str, public: bool, payload: List[int],
                 arguments: List[str], inputs: List[FunctionInput],
                 outputs: List[FunctionOutput]):
        self.request = "register_function"
        self.metadata = metadata
        self.name = name
        self.description = description
        self.executor_type = executor_type
        self.public = public
        self.payload = payload
        self.arguments = arguments
        self.inputs = inputs
        self.outputs = outputs


class RegisterInputFileRequest:
    def __init__(self, metadata: Metadata, url: str, cmac: str,
                 crypto_info: CryptoInfo):
        self.request = "register_input_file"
        self.metadata = metadata
        self.url = url
        self.cmac = cmac
        self.crypto_info = crypto_info


class RegisterOutputFileRequest:
    def __init__(self, metadata: Metadata, url: str, crypto_info: CryptoInfo):
        self.request = "register_output_file"
        self.metadata = metadata
        self.url = url
        self.crypto_info = crypto_info


class UpdateInputFileRequest:
    def __init__(self, metadata: Metadata, data_id: str, url: str):
        self.request = "update_input_file"
        self.metadata = metadata
        self.data_id = data_id
        self.url = url


class UpdateOutputFileRequest:
    def __init__(self, metadata: Metadata, data_id: str, url: str):
        self.request = "update_output_file"
        self.metadata = metadata
        self.data_id = data_id
        self.url = url


class CreateTaskRequest:
    def __init__(self, metadata: Metadata, function_id: str,
                 function_arguments: Dict[str, Any], executor: str,
                 inputs_ownership: List[OwnerList],
                 outputs_ownership: List[OwnerList]):
        self.request = "create_task"
        self.metadata = metadata
        self.function_id = function_id
        self.function_arguments = function_arguments
        self.executor = executor
        self.inputs_ownership = inputs_ownership
        self.outputs_ownership = outputs_ownership


class AssignDataRequest:
    def __init__(self, metadata: Metadata, task_id: str, inputs: List[DataMap],
                 outputs: List[DataMap]):
        self.request = "assign_data"
        self.metadata = metadata
        self.task_id = task_id
        self.inputs = inputs
        self.outputs = outputs


class ApproveTaskRequest:
    def __init__(self, metadata: Metadata, task_id: str):
        self.request = "approve_task"
        self.metadata = metadata
        self.task_id = task_id


class InvokeTaskRequest:
    def __init__(self, metadata: Metadata, task_id: str):
        self.request = "invoke_task"
        self.metadata = metadata
        self.task_id = task_id


class GetTaskRequest:
    def __init__(self, metadata: Metadata, task_id: str):
        self.request = "get_task"
        self.metadata = metadata
        self.task_id = task_id


class FrontendClient:
    def __init__(self, channel: ssl.SSLSocket, metadata: Metadata = None):
        self.channel = channel
        self.metadata = metadata

    def register_function(self,
                          name: str,
                          description: str,
                          executor_type: str,
                          public: bool = True,
                          payload: List[int] = [],
                          arguments: List[str] = [],
                          inputs: List[FunctionInput] = [],
                          outputs: List[FunctionOutput] = []):
        request = RegisterFunctionRequest(self.metadata, name, description,
                                          executor_type, public, payload,
                                          arguments, inputs, outputs)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        return response["content"]["function_id"]

    def register_input_file(self, url: str, schema: str, key: List[int],
                            iv: List[int], cmac: str):
        request = RegisterInputFileRequest(self.metadata, url, cmac,
                                           CryptoInfo(schema, key, iv))
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        return response["content"]["data_id"]

    def register_output_file(self, url: str, schema: str, key: List[int],
                             iv: List[int]):
        request = RegisterOutputFileRequest(self.metadata, url,
                                            CryptoInfo(schema, key, iv))
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        return response["content"]["data_id"]

    def create_task(self,
                    function_id: str,
                    function_arguments: Dict[str, Any],
                    executor: str,
                    inputs_ownership: List[OwnerList] = [],
                    outputs_ownership: List[OwnerList] = []):
        function_arguments = json.dumps(function_arguments)
        request = CreateTaskRequest(self.metadata, function_id,
                                    function_arguments, executor,
                                    inputs_ownership, outputs_ownership)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        return response["content"]["task_id"]

    def assign_data_to_task(self, task_id: str, inputs: List[DataMap],
                            outputs: List[DataMap]):
        request = AssignDataRequest(self.metadata, task_id, inputs, outputs)
        _write_message(self.channel, request)
        _ = _read_message(self.channel)
        return

    def approve_task(self, task_id: str):
        request = ApproveTaskRequest(self.metadata, task_id)
        _write_message(self.channel, request)
        _ = _read_message(self.channel)
        return

    def invoke_task(self, task_id: str):
        request = InvokeTaskRequest(self.metadata, task_id)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        assert (response["result"] == "ok")

    def get_task_result(self, task_id: str):
        request = GetTaskRequest(self.metadata, task_id)

        while True:
            _write_message(self.channel, request)
            response = _read_message(self.channel)
            time.sleep(1)
            if response["content"]["status"] == 10:
                break

        return response["content"]["result"]["result"]["Ok"]["return_value"]

    def get_output_cmac_by_tag(self, task_id: str, tag: str):
        request = GetTaskRequest(self.metadata, task_id)
        while True:
            _write_message(self.channel, request)
            response = _read_message(self.channel)
            time.sleep(1)
            if response["content"]["status"] == 10:
                break
        return response["content"]["result"]["result"]["Ok"]["tags_map"][tag]


def _write_message(sock: ssl.SSLSocket, message: Any):
    class RequestEncoder(json.JSONEncoder):
        def default(self, o):
            return o.__dict__

    message = json.dumps(message, cls=RequestEncoder).encode()
    sock.sendall(struct.pack(">Q", len(message)))
    sock.sendall(message)


def _read_message(sock: ssl.SSLSocket):
    response_len = struct.unpack(">Q", sock.read(8))
    raw = bytearray()
    total_recv = 0
    while total_recv < response_len[0]:
        data = sock.recv()
        total_recv += len(data)
        raw += data
    response = json.loads(raw)
    return response


def _verify_report(as_root_ca_cert_path: str, enclave_info_path: str,
                   cert: Dict[str, Any], endpoint_name: str):
    if os.environ.get('SGX_MODE') == 'SW':
        return

    cert = x509.load_der_x509_certificate(cert, default_backend())
    ext = json.loads(cert.extensions[0].value.value)

    report = bytes(ext["report"])
    signature = bytes(ext["signature"])
    signing_cert = bytes(ext["signing_cert"])
    signing_cert = load_certificate(FILETYPE_ASN1, signing_cert)

    # verify signing cert with AS root cert
    with open(as_root_ca_cert_path) as f:
        as_root_ca_cert = f.read()
    as_root_ca_cert = load_certificate(FILETYPE_PEM, as_root_ca_cert)
    store = X509Store()
    store.add_cert(as_root_ca_cert)
    store.add_cert(signing_cert)
    store_ctx = X509StoreContext(store, as_root_ca_cert)
    store_ctx.verify_certificate()

    # verify report's signature
    crypto.verify(signing_cert, signature, bytes(ext["report"]), 'sha256')

    report = json.loads(report)
    quote = report['isvEnclaveQuoteBody']
    quote = base64.b64decode(quote)

    # get mr_enclave and mr_signer from the quote
    mr_enclave = quote[112:112 + 32].hex()
    mr_signer = quote[176:176 + 32].hex()

    # get enclave_info
    enclave_info = toml.load(enclave_info_path)

    # verify mr_enclave and mr_signer
    enclave_name = "teaclave_" + endpoint_name + "_service"
    if mr_enclave != enclave_info[enclave_name]["mr_enclave"]:
        raise Exception("mr_enclave error")

    if mr_signer != enclave_info[enclave_name]["mr_signer"]:
        raise Exception("mr_signer error")
