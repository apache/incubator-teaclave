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
from enum import IntEnum

import cryptography
from cryptography import x509
from cryptography.hazmat.backends import default_backend

from OpenSSL.crypto import load_certificate, FILETYPE_PEM, FILETYPE_ASN1
from OpenSSL.crypto import X509Store, X509StoreContext
from OpenSSL import crypto

__all__ = [
    'FrontendService', 'AuthenticationService', 'FunctionArgument',
    'FunctionInput', 'FunctionOutput', 'OwnerList', 'DataMap'
]

Metadata = Dict[str, str]


class TaskStatus(IntEnum):
    Created = 0
    DataAssigned = 1
    Approved = 2
    Staged = 3
    Running = 4
    Finished = 10
    Canceled = 20
    Failed = 99


class Request:
    pass


class TeaclaveException(Exception):
    pass


class TeaclaveService:
    channel = None
    metadata = None

    def __init__(self,
                 name: str,
                 address: Tuple[str, int],
                 as_root_ca_cert_path: str,
                 enclave_info_path: str,
                 dump_report=False):
        self._context = ssl._create_unverified_context()
        self._name = name
        self._address = address
        self._as_root_ca_cert_path = as_root_ca_cert_path
        self._enclave_info_path = enclave_info_path
        self._closed = False
        self._dump_report = dump_report

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        if not self._closed:
            self.close()

    def close(self):
        self._closed = True
        if self.channel: self.channel.close()

    def check_channel(self):
        if not self.channel: raise TeaclaveException("Channel is None")

    def check_metadata(self):
        if not self.metadata: raise TeaclaveException("Metadata is None")

    def connect(self):
        """Establish trusted connection and verify remote attestation report.
        """
        sock = socket.create_connection(self._address)
        channel = self._context.wrap_socket(sock,
                                            server_hostname=self._address[0])
        cert = channel.getpeercert(binary_form=True)
        if not cert: raise TeaclaveException("Peer cert is None")
        try:
            self._verify_report(self._as_root_ca_cert_path,
                                self._enclave_info_path, cert, self._name)
        except Exception as e:
            raise TeaclaveException(
                f"Failed to verify attestation report: {e}")
        self.channel = channel

        return self

    def _verify_report(self, as_root_ca_cert_path: str, enclave_info_path: str,
                       cert: Dict[str, Any], endpoint_name: str):

        def load_certificates(pem_bytes):
            start_line = b'-----BEGIN CERTIFICATE-----'
            result = []
            cert_slots = pem_bytes.split(start_line)
            for single_pem_cert in cert_slots[1:]:
                cert = load_certificate(FILETYPE_ASN1,
                                        start_line + single_pem_cert)
                result.append(cert)
            return result

        if os.environ.get('SGX_MODE') == 'SW':
            return

        cert = x509.load_der_x509_certificate(cert, default_backend())

        if self._dump_report:
            try:
                with open(self._name + "_attestation_report.pem", "wb") as f:
                    f.write(
                        cert.public_bytes(cryptography.hazmat.primitives.
                                          serialization.Encoding.PEM))
            except:
                raise TeaclaveException("Failed to dump attestation report")

        try:
            ext = json.loads(cert.extensions[0].value.value)
        except:
            raise TeaclaveException("Failed to load extensions")

        report = bytes(ext["report"])
        signature = bytes(ext["signature"])
        try:
            certs = [
                load_certificate(FILETYPE_ASN1, bytes(c)) for c in ext["certs"]
            ]
        except:
            raise TeaclaveException(
                "Failed to load singing certificate of the report")

        # verify signing cert with AS root cert
        try:
            with open(as_root_ca_cert_path) as f:
                as_root_ca_cert = f.read()
        except:
            raise TeaclaveException(
                "Failed to open attestation service root certificate")

        try:
            as_root_ca_cert = load_certificate(FILETYPE_PEM, as_root_ca_cert)
        except:
            raise TeaclaveException(
                "Failed to load attestation service root certificate")

        store = X509Store()
        store.add_cert(as_root_ca_cert)
        client_cert = certs[0]
        if len(certs) > 1:
            for c in certs[1:]:
                store.add_cert(c)
        store_ctx = X509StoreContext(store, client_cert)

        try:
            store_ctx.verify_certificate()

            # verify report's signature
            crypto.verify(certs[0], signature, bytes(ext["report"]), 'sha256')
        except:
            raise TeaclaveException("Failed to verify report signature")

        report = json.loads(report)
        quote = report['isvEnclaveQuoteBody']
        quote = base64.b64decode(quote)

        # get report_data from the quote
        report_data = quote[368:368 + 64]
        # get EC pub key from the certificate
        pub_key = cert.public_key().public_bytes(
            cryptography.hazmat.primitives.serialization.Encoding.X962,
            cryptography.hazmat.primitives.serialization.PublicFormat.
            UncompressedPoint)

        # verify whether the certificate is bound to the quote
        assert (pub_key[0] == 4)
        if pub_key[1:] != report_data:
            raise TeaclaveException(
                "Failed to verify the certificate agaist the report data in the quote"
            )

        # get mr_enclave and mr_signer from the quote
        mr_enclave = quote[112:112 + 32].hex()
        mr_signer = quote[176:176 + 32].hex()

        # get enclave_info
        try:
            enclave_info = toml.load(enclave_info_path)
        except:
            raise TeaclaveException("Failed to load enclave info")

        # verify mr_enclave and mr_signer
        enclave_name = "teaclave_" + endpoint_name + "_service"
        if mr_enclave != enclave_info[enclave_name]["mr_enclave"]:
            raise Exception("Failed to verify mr_enclave")

        if mr_signer != enclave_info[enclave_name]["mr_signer"]:
            raise Exception("Failed to verify mr_signer")


class FunctionInput:
    """Function input for registering.

    Args:

        name: Name of input data.
        description: Description of the input data.
        optional: [Default: False] Data owners do not need to register the data.
    """

    def __init__(self, name: str, description: str, optional=False):
        self.name = name
        self.description = description
        self.optional = optional


class FunctionOutput:
    """Function output for registering.

    Args:

        name: Name of output data.
        description: Description of the output data.
        optional: [Default: False] Data owners do not need to register the data.
    """

    def __init__(self, name: str, description: str, optional=False):
        self.name = name
        self.description = description
        self.optional = optional


class FunctionArgument:
    """Function argument for registring.

    Args:
        key: Name of the argument.
        default_value: A default value of the argument. The default value is "".
        allow_overwrite: If allow_overwrite flag is set to be true. The service
                         will allow the task creator to overwrite the arguement
                         value when creating tasks.
    """

    def __init__(self,
                 key: str,
                 default_value: str = "",
                 allow_overwrite=True):
        self.key = key
        self.default_value = default_value
        self.allow_overwrite = allow_overwrite


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


class UserRegisterRequest(Request):

    def __init__(self, metadata: Metadata, user_id: str, user_password: str,
                 role: str, attribute: str):
        self.request = "user_register"
        self.metadata = metadata
        self.id = user_id
        self.password = user_password
        self.role = role
        self.attribute = attribute


class UserUpdateRequest(Request):

    def __init__(self, metadata: Metadata, user_id: str, user_password: str,
                 role: str, attribute: str):
        self.request = "user_update"
        self.metadata = metadata
        self.id = user_id
        self.password = user_password
        self.role = role
        self.attribute = attribute


class UserLoginRequest(Request):

    def __init__(self, user_id: str, user_password: str):
        self.request = "user_login"
        self.id = user_id
        self.password = user_password


class UserChangePasswordRequest(Request):

    def __init__(self, metadata: Metadata, password: str):
        self.request = "user_change_password"
        self.metadata = metadata
        self.password = password


class ResetUserPasswordRequest(Request):

    def __init__(self, metadata: Metadata, user_id: str):
        self.request = "reset_user_password"
        self.metadata = metadata
        self.id = user_id


class DeleteUserRequest(Request):

    def __init__(self, metadata: Metadata, user_id: str):
        self.request = "delete_user"
        self.metadata = metadata
        self.id = user_id


class ListUsersRequest(Request):

    def __init__(self, metadata: Metadata, user_id: str):
        self.request = "list_users"
        self.metadata = metadata
        self.id = user_id


class AuthenticationService(TeaclaveService):
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

    def __init__(self,
                 address: Tuple[str, int],
                 as_root_ca_cert_path: str,
                 enclave_info_path: str,
                 dump_report=False):
        super().__init__("authentication", address, as_root_ca_cert_path,
                         enclave_info_path, dump_report)

    def user_register(self, user_id: str, user_password: str, role: str,
                      attribute: str):
        """Register a new user.

        Args:

            user_id: User ID.
            user_password: Password.
            role: Role of user.
            attribute: Attribute related to the role.
        """
        self.check_channel()
        self.check_metadata()
        request = UserRegisterRequest(self.metadata, user_id, user_password,
                                      role, attribute)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            pass
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(f"Failed to register user ({reason})")

    def user_update(self, user_id: str, user_password: str, role: str,
                    attribute: str):
        """Update an existing user.

        Args:

            user_id: User ID.
            user_password: Password.
            role: Role of user.
            attribute: Attribute related to the role.
        """
        self.check_channel()
        self.check_metadata()
        request = UserUpdateRequest(self.metadata, user_id, user_password,
                                    role, attribute)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            pass
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(f"Failed to update user ({reason})")

    def user_login(self, user_id: str, user_password: str) -> str:
        """Login and get a session token.

        Args:

            user_id: User ID.
            user_password: Password.

        Returns:

            str: User login token.
        """
        self.check_channel()
        request = UserLoginRequest(user_id, user_password)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            return response["content"]["token"]
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(f"Failed to login user ({reason})")

    def user_change_password(self, user_password: str):
        """Change password.

        Args:

            user_password: New password.
        """
        self.check_channel()
        self.check_metadata()
        request = UserChangePasswordRequest(self.metadata, user_password)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            pass
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(f"Failed to change password ({reason})")

    def reset_user_password(self, user_id: str) -> str:
        """Reset password of a managed user.

        Args:

            user_id: User ID.

        Returns:

            str: New password.
        """
        self.check_channel()
        self.check_metadata()
        request = ResetUserPasswordRequest(self.metadata, user_id)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            return response["content"]["password"]
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(f"Failed to reset password ({reason})")

    def delete_user(self, user_id: str) -> str:
        """Delete a user.

        Args:

            user_id: User ID.
        """
        self.check_channel()
        self.check_metadata()
        request = DeleteUserRequest(self.metadata, user_id)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            pass
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(f"Failed to delete user ({reason})")

    def list_users(self, user_id: str) -> str:
        """List managed users

        Args:

            user_id: User ID.

        Returns:

            str: User list
        """
        self.check_channel()
        request = ListUsersRequest(self.metadata, user_id)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            return response["content"]["ids"]
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(f"Failed to list user ({reason})")


class RegisterFunctionRequest(Request):

    def __init__(self, metadata: Metadata, name: str, description: str,
                 executor_type: str, public: bool, payload: List[int],
                 arguments: List[FunctionArgument],
                 inputs: List[FunctionInput], outputs: List[FunctionOutput],
                 user_allowlist: List[str], usage_quota: int):
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
        self.user_allowlist = user_allowlist
        self.usage_quota = usage_quota


class UpdateFunctionRequest(Request):

    def __init__(self, metadata: Metadata, function_id: str, name: str,
                 description: str, executor_type: str, public: bool,
                 payload: List[int], arguments: List[FunctionArgument],
                 inputs: List[FunctionInput], outputs: List[FunctionOutput],
                 user_allowlist: List[str], usage_quota: int):
        self.request = "update_function"
        self.metadata = metadata
        self.function_id = function_id
        self.name = name
        self.description = description
        self.executor_type = executor_type
        self.public = public
        self.payload = payload
        self.arguments = arguments
        self.inputs = inputs
        self.outputs = outputs
        self.user_allowlist = user_allowlist
        self.usage_quota = usage_quota


class ListFunctionsRequest(Request):

    def __init__(self, metadata: Metadata, user_id: str):
        self.request = "list_functions"
        self.metadata = metadata
        self.user_id = user_id


class DeleteFunctionRequest(Request):

    def __init__(self, metadata: Metadata, function_id: str):
        self.request = "delete_function"
        self.metadata = metadata
        self.function_id = function_id


class DisableFunctionRequest(Request):

    def __init__(self, metadata: Metadata, function_id: str):
        self.request = "disable_function"
        self.metadata = metadata
        self.function_id = function_id


class GetFunctionRequest(Request):

    def __init__(self, metadata: Metadata, function_id: str):
        self.request = "get_function"
        self.metadata = metadata
        self.function_id = function_id


class GetFunctionUsageStatsRequest(Request):

    def __init__(self, metadata: Metadata, function_id: str):
        self.request = "get_function_usage_stats"
        self.metadata = metadata
        self.function_id = function_id


class RegisterInputFileRequest(Request):

    def __init__(self, metadata: Metadata, url: str, cmac: List[int],
                 crypto_info: CryptoInfo):
        self.request = "register_input_file"
        self.metadata = metadata
        self.url = url
        self.cmac = cmac
        self.crypto_info = crypto_info


class RegisterOutputFileRequest(Request):

    def __init__(self, metadata: Metadata, url: str, crypto_info: CryptoInfo):
        self.request = "register_output_file"
        self.metadata = metadata
        self.url = url
        self.crypto_info = crypto_info


class UpdateInputFileRequest(Request):

    def __init__(self, metadata: Metadata, data_id: str, url: str):
        self.request = "update_input_file"
        self.metadata = metadata
        self.data_id = data_id
        self.url = url


class UpdateOutputFileRequest(Request):

    def __init__(self, metadata: Metadata, data_id: str, url: str):
        self.request = "update_output_file"
        self.metadata = metadata
        self.data_id = data_id
        self.url = url


class CreateTaskRequest(Request):

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


class AssignDataRequest(Request):

    def __init__(self, metadata: Metadata, task_id: str, inputs: List[DataMap],
                 outputs: List[DataMap]):
        self.request = "assign_data"
        self.metadata = metadata
        self.task_id = task_id
        self.inputs = inputs
        self.outputs = outputs


class ApproveTaskRequest(Request):

    def __init__(self, metadata: Metadata, task_id: str):
        self.request = "approve_task"
        self.metadata = metadata
        self.task_id = task_id


class InvokeTaskRequest(Request):

    def __init__(self, metadata: Metadata, task_id: str):
        self.request = "invoke_task"
        self.metadata = metadata
        self.task_id = task_id


class CancelTaskRequest(Request):

    def __init__(self, metadata: Metadata, task_id: str):
        self.request = "cancel_task"
        self.metadata = metadata
        self.task_id = task_id


class GetTaskRequest(Request):

    def __init__(self, metadata: Metadata, task_id: str):
        self.request = "get_task"
        self.metadata = metadata
        self.task_id = task_id


class FrontendService(TeaclaveService):
    """Establish trusted channel with the frontend service and provide
    clients to send request through RPC.

    Args:
    
        address: The address of the remote services in tuple.
        as_root_ca_cert_path: Root CA certification of the attestation services
            to verify the attestation report.
        enclave_info_path: Path of enclave info to verify the remote service in
            the attestation report.
    """

    def __init__(self,
                 address: Tuple[str, int],
                 as_root_ca_cert_path: str,
                 enclave_info_path: str,
                 dump_report=False):
        super().__init__("frontend", address, as_root_ca_cert_path,
                         enclave_info_path, dump_report)

    def register_function(
        self,
        name: str,
        description: str,
        executor_type: str,
        public: bool = True,
        payload: List[int] = [],
        arguments: List[FunctionArgument] = [],
        inputs: List[FunctionInput] = [],
        outputs: List[FunctionOutput] = [],
        user_allowlist: List[str] = [],
        usage_quota: int = -1,
    ):
        self.check_metadata()
        self.check_channel()
        request = RegisterFunctionRequest(self.metadata, name, description,
                                          executor_type, public, payload,
                                          arguments, inputs, outputs,
                                          user_allowlist, usage_quota)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            return response["content"]["function_id"]
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(f"Failed to register function ({reason})")

    def update_function(
        self,
        function_id: str,
        name: str,
        description: str,
        executor_type: str,
        public: bool = True,
        payload: List[int] = [],
        arguments: List[FunctionArgument] = [],
        inputs: List[FunctionInput] = [],
        outputs: List[FunctionOutput] = [],
        user_allowlist: List[str] = [],
        usage_quota: int = -1,
    ):
        self.check_metadata()
        self.check_channel()
        request = UpdateFunctionRequest(self.metadata, function_id, name,
                                        description, executor_type, public,
                                        payload, arguments, inputs, outputs,
                                        user_allowlist, usage_quota)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            return response["content"]["function_id"]
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(f"Failed to update function ({reason})")

    def list_functions(self, user_id: str):
        self.check_metadata()
        self.check_channel()
        request = ListFunctionsRequest(self.metadata, user_id)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            return response["content"]
        else:
            raise TeaclaveException("Failed to list functions")

    def get_function(self, function_id: str):
        self.check_metadata()
        self.check_channel()
        request = GetFunctionRequest(self.metadata, function_id)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            return response["content"]
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(f"Failed to get function ({reason})")

    def get_function_usage_stats(self, user_id: str, function_id: str):
        self.check_metadata()
        self.check_channel()
        request = GetFunctionUsageStatsRequest(self.metadata, function_id)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            return response["content"]
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(
                f"Failed to get function usage statistics ({reason})")

    def delete_function(self, function_id: str):
        self.check_metadata()
        self.check_channel()
        request = DeleteFunctionRequest(self.metadata, function_id)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            return response["content"]
        else:
            raise TeaclaveException("Failed to delete function")

    def disable_function(self, function_id: str):
        self.check_metadata()
        self.check_channel()
        request = DisableFunctionRequest(self.metadata, function_id)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            return response["content"]
        else:
            raise TeaclaveException("Failed to disable function")

    def register_input_file(self, url: str, schema: str, key: List[int],
                            iv: List[int], cmac: List[int]):
        self.check_metadata()
        self.check_channel()
        request = RegisterInputFileRequest(self.metadata, url, cmac,
                                           CryptoInfo(schema, key, iv))
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            return response["content"]["data_id"]
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(
                f"Failed to register input file ({reason})")

    def register_output_file(self, url: str, schema: str, key: List[int],
                             iv: List[int]):
        self.check_metadata()
        self.check_channel()
        request = RegisterOutputFileRequest(self.metadata, url,
                                            CryptoInfo(schema, key, iv))
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            return response["content"]["data_id"]
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(
                f"Failed to register output file ({reason})")

    def create_task(self,
                    function_id: str,
                    function_arguments: Dict[str, Any],
                    executor: str,
                    inputs_ownership: List[OwnerList] = [],
                    outputs_ownership: List[OwnerList] = []):
        self.check_metadata()
        self.check_channel()
        function_arguments = json.dumps(function_arguments)
        request = CreateTaskRequest(self.metadata, function_id,
                                    function_arguments, executor,
                                    inputs_ownership, outputs_ownership)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            return response["content"]["task_id"]
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(f"Failed to create task ({reason})")

    def assign_data_to_task(self, task_id: str, inputs: List[DataMap],
                            outputs: List[DataMap]):
        self.check_metadata()
        self.check_channel()
        request = AssignDataRequest(self.metadata, task_id, inputs, outputs)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            pass
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(
                f"Failed to assign data to task ({reason})")

    def approve_task(self, task_id: str):
        self.check_metadata()
        self.check_channel()
        request = ApproveTaskRequest(self.metadata, task_id)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            pass
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(f"Failed to approve task ({reason})")

    def invoke_task(self, task_id: str):
        self.check_metadata()
        self.check_channel()
        request = InvokeTaskRequest(self.metadata, task_id)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            pass
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(f"Failed to invoke task ({reason})")

    def cancel_task(self, task_id: str):
        self.check_metadata()
        self.check_channel()
        request = CancelTaskRequest(self.metadata, task_id)
        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] == "ok":
            pass
        else:
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(f"Failed to cancel task ({reason})")

    def get_task(self, task_id: str) -> dict:
        self.check_metadata()
        self.check_channel()
        request = GetTaskRequest(self.metadata, task_id)

        _write_message(self.channel, request)
        response = _read_message(self.channel)
        if response["result"] != "ok":
            reason = "unknown"
            if "request_error" in response:
                reason = response["request_error"]
            raise TeaclaveException(f"Failed to get task result ({reason})")
        return response["content"]

    def get_task_result(self, task_id: str):
        self.check_metadata()
        self.check_channel()
        request = GetTaskRequest(self.metadata, task_id)

        while True:
            _write_message(self.channel, request)
            response = _read_message(self.channel)
            if response["result"] != "ok":
                reason = "unknown"
                if "request_error" in response:
                    reason = response["request_error"]
                raise TeaclaveException(
                    f"Failed to get task result ({reason})")
            time.sleep(1)
            if response["content"]["status"] == TaskStatus.Finished:
                break
            elif response["content"]["status"] == TaskStatus.Canceled:
                raise TeaclaveException(
                    "Task Canceled, Error: " +
                    response["content"]["result"]["result"]["Err"]["reason"])
            elif response["content"]["status"] == TaskStatus.Failed:
                raise TeaclaveException(
                    "Task Failed, Error: " +
                    response["content"]["result"]["result"]["Err"]["reason"])

        return response["content"]["result"]["result"]["Ok"]["return_value"]

    def get_output_cmac_by_tag(self, task_id: str, tag: str):
        self.check_metadata()
        self.check_channel()
        request = GetTaskRequest(self.metadata, task_id)
        while True:
            _write_message(self.channel, request)
            response = _read_message(self.channel)
            if response["result"] != "ok":
                reason = "unknown"
                if "request_error" in response:
                    reason = response["request_error"]
                raise TeaclaveException(
                    f"Failed to get output cmac by tag ({reason})")
            time.sleep(1)
            if response["content"]["status"] == TaskStatus.Finished:
                break

        return response["content"]["result"]["result"]["Ok"]["tags_map"][tag]


def _write_message(sock: ssl.SSLSocket, message: Any):

    class RequestEncoder(json.JSONEncoder):

        def default(self, o):
            if isinstance(o, Request):
                request = o.__dict__["request"]
                j = {}
                j["message"] = {}
                j["message"][request] = {}
                for k, v in o.__dict__.items():
                    if k == "metadata": j[k] = v
                    elif k == "request": continue
                    else: j["message"][request][k] = v
                return j
            else:
                return o.__dict__

    message = json.dumps(message, cls=RequestEncoder,
                         separators=(',', ':')).encode()
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
