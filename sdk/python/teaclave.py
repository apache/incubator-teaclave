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

import json
import base64
import toml
import time
import os
import ssl

import cryptography
from cryptography import x509
from cryptography.hazmat.backends import default_backend

from google.protobuf.json_format import MessageToDict
from google.protobuf.empty_pb2 import Empty
from grpclib.client import Channel, _ChannelState
from grpclib.protocol import H2Protocol

from OpenSSL.crypto import load_certificate, FILETYPE_PEM, FILETYPE_ASN1
from OpenSSL.crypto import X509Store, X509StoreContext
from OpenSSL import crypto

import teaclave_authentication_service_pb2 as auth
import teaclave_frontend_service_pb2 as fe
from teaclave_authentication_service_grpc import TeaclaveAuthenticationApiStub
from teaclave_frontend_service_grpc import TeaclaveFrontendStub
from teaclave_common_pb2 import TaskStatus, FileCryptoInfo

from typing import Tuple, Dict, List, Any

__all__ = [
    'FrontendService', 'AuthenticationService', 'FunctionArgument',
    'FunctionInput', 'FunctionOutput', 'OwnerList', 'DataMap'
]

Metadata = Dict[str, str]


class Request:
    message = None

    def __init__(self, method, response, metadata=dict()):
        self.method = method
        self.metadata = metadata
        self.response = response


class TeaclaveException(Exception):
    pass


class TeaclaveService:
    metadata = None
    stub = None

    def __init__(self,
                 name: str,
                 address: Tuple[str, int],
                 as_root_ca_cert_path: str,
                 enclave_info_path: str,
                 dump_report=False):
        self._name = name
        self._address = address
        self._as_root_ca_cert_path = as_root_ca_cert_path
        self._enclave_info_path = enclave_info_path
        self._dump_report = dump_report

        self._channel = TeaclaveChannel(self._name, self._address,
                                        self._as_root_ca_cert_path,
                                        self._enclave_info_path)
        self._loop = self._channel._loop

    def call_method(self, request):
        return self._loop.run_until_complete(
            getattr(self.stub, request.method)(request.message,
                                               metadata=request.metadata))

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        self.close()

    def close(self):
        if self._channel: self._channel.close()

    def __del__(self) -> None:
        self.close()

    def check_metadata(self):
        if not self.metadata: raise TeaclaveException("Metadata is None")

    def check_channel(self):
        self._channel.check_channel()

    def get_metadata(self):
        return self.metadata


def create_context() -> ssl.SSLContext:
    ctx = ssl._create_unverified_context()
    ctx.options |= ssl.OP_NO_TLSv1 | ssl.OP_NO_TLSv1_1
    ctx.set_ciphers('ECDHE+AESGCM:ECDHE+CHACHA20:DHE+AESGCM:DHE+CHACHA20')
    ctx.set_alpn_protocols(['h2'])
    try:
        ctx.set_npn_protocols(['h2'])
    except NotImplementedError:
        pass
    return ctx


class TeaclaveChannel(Channel):

    def __init__(self,
                 name: str,
                 address: Tuple[str, int],
                 as_root_ca_cert_path: str,
                 enclave_info_path: str,
                 dump_report=False):
        context = create_context()
        super().__init__(host=address[0], port=address[1], ssl=context)
        self._name = name
        self._as_root_ca_cert_path = as_root_ca_cert_path
        self._enclave_info_path = enclave_info_path
        self._dump_report = dump_report

    def check_channel(self):
        if self._state == _ChannelState.TRANSIENT_FAILURE:
            raise TeaclaveException("Channel is None")

    async def __connect__(self) -> H2Protocol:
        protocol = await super().__connect__()
        sslobj = protocol.connection._transport.get_extra_info('ssl_object')
        cert = sslobj.getpeercert(binary_form=True)
        if not cert: raise TeaclaveException("Peer cert is None")
        try:
            self._verify_report(self._as_root_ca_cert_path,
                                self._enclave_info_path, cert, self._name)
        except Exception as e:
            raise TeaclaveException(
                f"Failed to verify attestation report: {e}")
        return protocol

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
        self.message = fe.FunctionInput(name=name,
                                        description=description,
                                        optional=optional)


class FunctionOutput:
    """Function output for registering.

    Args:

        name: Name of output data.
        description: Description of the output data.
        optional: [Default: False] Data owners do not need to register the data.
    """

    def __init__(self, name: str, description: str, optional=False):
        self.message = fe.FunctionOutput(name=name,
                                         description=description,
                                         optional=optional)


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
        self.message = fe.FunctionArgument(key=key,
                                           default_value=default_value,
                                           allow_overwrite=allow_overwrite)


class OwnerList:
    """Defines data ownership.

    Args:

        data_name: Name of output data.
        uids: A list of user id which own this data.
    """

    def __init__(self, data_name: str, uids: List[str]):
        self.message = fe.OwnerList(data_name=data_name, uids=uids)


class DataMap:
    """Assign data id to input or output data.

    Args:

        data_name: Name of output data.
        data_id: Id for the data name.
    """

    def __init__(self, data_name, data_id):
        self.message = fe.DataMap(data_name=data_name, data_id=data_id)


class CryptoInfo:
    """Cryptographic information for the input/output data.

    Args:

        schema: Encryption algorithms for the input/output data.
        key: Key for encryption and decryption, bytes in list.
        iv: IV, bytes in list.
    """

    def __init__(self, schema: str, key: List[int], iv: List[int]):

        self.message = FileCryptoInfo(schema=schema,
                                      key=bytes(key),
                                      iv=bytes(iv))


class UserRegisterRequest(Request):

    def __init__(self, metadata: Metadata, user_id: str, user_password: str,
                 role: str, attribute: str):
        super().__init__("UserRegister", Empty, metadata)
        self.message = auth.UserRegisterRequest(id=user_id,
                                                password=user_password,
                                                role=role,
                                                attribute=attribute)


class UserUpdateRequest(Request):

    def __init__(self, metadata: Metadata, user_id: str, user_password: str,
                 role: str, attribute: str):
        super().__init__("UserUpdate", Empty)
        self.message = auth.UserUpdateRequest(id=user_id,
                                              password=user_password,
                                              role=role,
                                              attribute=attribute)


class UserLoginRequest(Request):

    def __init__(self, user_id: str, user_password: str):
        super().__init__("UserLogin", auth.UserLoginResponse)
        self.message = auth.UserLoginRequest(id=user_id,
                                             password=user_password)


class UserChangePasswordRequest(Request):

    def __init__(self, metadata: Metadata, password: str):
        super().__init__("UserChangePassword", Empty, metadata)
        self.message = auth.UserChangePasswordRequest(password=password)


class ResetUserPasswordRequest(Request):

    def __init__(self, metadata: Metadata, user_id: str):
        super().__init__("ResetUserPassword", auth.ResetUserPasswordResponse,
                         metadata)
        self.message = auth.ResetUserPasswordRequest(id=user_id)


class DeleteUserRequest(Request):

    def __init__(self, metadata: Metadata, user_id: str):
        super().__init__("DeleteUser", Empty, metadata)
        self.message = auth.DeleteUserRequest(id=user_id)


class ListUsersRequest(Request):

    def __init__(self, metadata: Metadata, user_id: str):
        super().__init__("ListUsers", auth.ListUsersResponse, metadata)
        self.message = auth.ListUsersRequest(id=user_id)


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
        self.stub = TeaclaveAuthenticationApiStub(self._channel)

    def user_register(self,
                      user_id: str,
                      user_password: str,
                      role="",
                      attribute=""):
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
        try:
            response = self.call_method(request)
            return response
        except Exception as e:
            raise TeaclaveException(f"Failed to register user  {str(e)}")

    def user_update(self,
                    user_id: str,
                    user_password: str,
                    role: str,
                    attribute=""):
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
        try:
            response = self.call_method(request)
            return response
        except Exception as e:
            raise TeaclaveException(f"Failed to update user  {str(e)}")

    def user_login(self, user_id: str, user_password: str) -> str:
        """Login and get a session token.

        Args:

            user_id: User ID.
            user_password: Password.

        Returns:

            str: User login token.
        """
        self._channel.check_channel()
        request = UserLoginRequest(user_id, user_password)
        try:
            response = self.call_method(request)
            self.metadata = {"id": user_id, "token": response.token}
            return response.token
        except Exception as e:
            raise TeaclaveException(f"Failed to login user  {str(e)}")

    def user_change_password(self, user_password: str):
        """Change password.

        Args:

            user_password: New password.
        """
        self.check_channel()
        self.check_metadata()
        request = UserChangePasswordRequest(self.metadata, user_password)
        try:
            response = self.call_method(request)
            return response
        except Exception as e:
            raise TeaclaveException(f"Failed to change password  {str(e)}")

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
        try:
            response = self.call_method(request)
            return response
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(f"Failed to reset password  {reason}")

    def delete_user(self, user_id: str) -> str:
        """Delete a user.

        Args:

            user_id: User ID.
        """
        self.check_channel()
        self.check_metadata()
        request = DeleteUserRequest(self.metadata, user_id)
        try:
            response = self.call_method(request)
            return response
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(f"Failed to delete user ({reason})")

    def list_users(self, user_id: str) -> str:
        """List managed users

        Args:

            user_id: User ID.

        Returns:

            str: User list
        """
        self.check_channel()
        self.check_metadata()
        request = ListUsersRequest(self.metadata, user_id)
        try:
            response = self.call_method(request)
            return response
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(f"Failed to list user ({reason})")


class RegisterFunctionRequest(Request):

    def __init__(self, metadata: Metadata, name: str, description: str,
                 executor_type: str, public: bool, payload: List[int],
                 arguments: List[FunctionArgument],
                 inputs: List[FunctionInput], outputs: List[FunctionOutput],
                 user_allowlist: List[str], usage_quota: int):
        super().__init__("RegisterFunction", fe.RegisterFunctionResponse,
                         metadata)
        arguments = [x.message for x in arguments]
        inputs = [x.message for x in inputs]
        outputs = [x.message for x in outputs]

        self.message = fe.RegisterFunctionRequest(
            name=name,
            description=description,
            executor_type=executor_type,
            public=public,
            payload=bytes(payload),
            arguments=arguments,
            inputs=inputs,
            outputs=outputs,
            user_allowlist=user_allowlist,
            usage_quota=usage_quota)


class UpdateFunctionRequest(Request):

    def __init__(self, metadata: Metadata, function_id: str, name: str,
                 description: str, executor_type: str, public: bool,
                 payload: List[int], arguments: List[FunctionArgument],
                 inputs: List[FunctionInput], outputs: List[FunctionOutput],
                 user_allowlist: List[str], usage_quota: int):
        super().__init__("UpdateFunction", fe.UpdateFunctionResponse, metadata)
        arguments = [x.message for x in arguments]
        inputs = [x.message for x in inputs]
        outputs = [x.message for x in outputs]

        self.message = fe.UpdateFunctionRequest(function_id, name, description,
                                                executor_type, public, payload,
                                                arguments, inputs, outputs,
                                                user_allowlist, usage_quota)


class ListFunctionsRequest(Request):

    def __init__(self, metadata: Metadata, user_id: str):
        super().__init__("ListFunctions", fe.ListFunctionsResponse, metadata)
        self.message = fe.ListFunctionsRequest(user_id=user_id)


class DeleteFunctionRequest(Request):

    def __init__(self, metadata: Metadata, function_id: str):
        super().__init__("ListFunctions", Empty, metadata)
        self.message = fe.DeleteFunctionRequest(function_id=function_id)


class DisableFunctionRequest(Request):

    def __init__(self, metadata: Metadata, function_id: str):
        super().__init__("DisableFunction", Empty, metadata)
        self.message = fe.DisableFunctionRequest(function_id=function_id)


class GetFunctionRequest(Request):

    def __init__(self, metadata: Metadata, function_id: str):
        super().__init__("GetFunction", fe.GetFunctionResponse, metadata)
        self.message = fe.GetFunctionRequest(function_id=function_id)


class GetFunctionUsageStatsRequest(Request):

    def __init__(self, metadata: Metadata, function_id: str):
        super().__init__("GetFunctionUsageStats",
                         fe.GetFunctionUsageStatsResponse, metadata)
        self.message = fe.GetFunctionUsageStatsRequest(function_id=function_id)


class RegisterInputFileRequest(Request):

    def __init__(self, metadata: Metadata, url: str, cmac: List[int],
                 crypto_info: CryptoInfo):
        super().__init__("RegisterInputFile", fe.RegisterInputFileResponse,
                         metadata)
        self.message = fe.RegisterInputFileRequest(
            url=url, cmac=bytes(cmac), crypto_info=crypto_info.message)


class RegisterOutputFileRequest(Request):

    def __init__(self, metadata: Metadata, url: str, crypto_info: CryptoInfo):
        super().__init__("RegisterOutputFile", fe.RegisterOutputFileResponse,
                         metadata)
        self.message = fe.RegisterOutputFileRequest(
            url=url, crypto_info=crypto_info.message)


class RegisterInputFromOutputRequest(Request):

    def __init__(self, metadata: Metadata, data_id: str):
        super().__init__("RegisterInputFromOutput",
                         fe.RegisterInputFromOutputResponse, metadata)
        self.message = fe.RegisterInputFromOutputRequest(data_id=data_id)


class RegisterFusionOutputRequest(Request):

    def __init__(self, metadata: Metadata, owner_list: List[str] = []):
        super().__init__("RegisterFusionOutput",
                         fe.RegisterFusionOutputResponse, metadata)
        self.message = fe.RegisterFusionOutputRequest(owner_list=owner_list)


class UpdateInputFileRequest(Request):

    def __init__(self, metadata: Metadata, data_id: str, url: str):
        super().__init__("UpdateInputFile", fe.UpdateInputFileResponse,
                         metadata)
        self.message = fe.UpdateInputFileRequest(data_id=data_id, url=url)


class UpdateOutputFileRequest(Request):

    def __init__(self, metadata: Metadata, data_id: str, url: str):
        super().__init__("UpdateInputFile", fe.UpdateOutputFileResponse,
                         metadata)
        self.message = fe.UpdateOutputFileRequest(data_id=data_id, url=url)


class CreateTaskRequest(Request):

    def __init__(self, metadata: Metadata, function_id: str,
                 function_arguments: Dict[str, Any], executor: str,
                 inputs_ownership: List[OwnerList],
                 outputs_ownership: List[OwnerList]):
        super().__init__("CreateTask", fe.CreateTaskResponse, metadata)
        inputs_ownership = [x.message for x in inputs_ownership]
        outputs_ownership = [x.message for x in outputs_ownership]

        self.message = fe.CreateTaskRequest(
            function_id=function_id,
            function_arguments=function_arguments,
            executor=executor,
            inputs_ownership=inputs_ownership,
            outputs_ownership=outputs_ownership)


class AssignDataRequest(Request):

    def __init__(self, metadata: Metadata, task_id: str, inputs: List[DataMap],
                 outputs: List[DataMap]):
        super().__init__("AssignData", Empty, metadata)
        inputs = [x.message for x in inputs]
        outputs = [x.message for x in outputs]
        self.message = fe.AssignDataRequest(task_id=task_id,
                                            inputs=inputs,
                                            outputs=outputs)


class ApproveTaskRequest(Request):

    def __init__(self, metadata: Metadata, task_id: str):
        super().__init__("ApproveTask", Empty, metadata)
        self.message = fe.ApproveTaskRequest(task_id=task_id)


class InvokeTaskRequest(Request):

    def __init__(self, metadata: Metadata, task_id: str):
        super().__init__("InvokeTask", Empty, metadata)
        self.message = fe.InvokeTaskRequest(task_id=task_id)


class CancelTaskRequest(Request):

    def __init__(self, metadata: Metadata, task_id: str):
        super().__init__("CancelTask", Empty, metadata)
        self.message = fe.CancelTaskRequest(task_id=task_id)


class GetTaskRequest(Request):

    def __init__(self, metadata: Metadata, task_id: str):
        super().__init__("GetTask", fe.GetTaskResponse, metadata)
        self.message = fe.GetTaskRequest(task_id=task_id)


class QueryAuditLogsRequest(Request):

    def __init__(self, metadata: Metadata, message: str, limit: int):
        super().__init__("QueryAuditLogs", fe.QueryAuditLogsResponse, metadata)
        self.message = fe.QueryAuditLogsReqeust(message=message, limit=limit)


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
        self.stub = TeaclaveFrontendStub(self._channel)

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
        try:
            response = self.call_method(request)
            return response.function_id
        except Exception as e:
            reason = str(e)
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
        try:
            response = self.call_method(request)
            return response.function_id
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(f"Failed to register function ({reason})")

    def list_functions(self, user_id: str):
        self.check_metadata()
        self.check_channel()
        request = ListFunctionsRequest(self.metadata, user_id)
        try:
            response = self.call_method(request)
        except Exception as e:
            raise TeaclaveException(f"Failed to list functions ({str(e)})")
        return MessageToDict(response,
                             preserving_proto_field_name=True,
                             use_integers_for_enums=True)

    def get_function(self, function_id: str):
        self.check_metadata()
        self.check_channel()
        request = GetFunctionRequest(self.metadata, function_id)
        try:
            response = self.call_method(request)
            return response
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(f"Failed to get function ({reason})")

    def get_function_usage_stats(self, function_id: str):
        self.check_metadata()
        self.check_channel()
        request = GetFunctionUsageStatsRequest(self.metadata, function_id)
        try:
            response = self.call_method(request)
            return response
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(
                f"Failed to get function usage statistics ({reason})")

    def delete_function(self, function_id: str):
        self.check_metadata()
        self.check_channel()
        request = DeleteFunctionRequest(self.metadata, function_id)
        try:
            response = self.call_method(request)
            return response
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(f"Failed to delete function ({reason})")

    def disable_function(self, function_id: str):
        self.check_metadata()
        self.check_channel()
        request = DisableFunctionRequest(self.metadata, function_id)
        try:
            response = self.call_method(request)
            return response
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(f"Failed to disable function ({reason})")

    def register_input_file(self, url: str, schema: str, key: List[int],
                            iv: List[int], cmac: List[int]):
        self.check_metadata()
        self.check_channel()
        request = RegisterInputFileRequest(self.metadata, url, cmac,
                                           CryptoInfo(schema, key, iv))
        try:
            response = self.call_method(request)
            return response.data_id
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(
                f"Failed to register input file ({reason})")

    def register_output_file(self, url: str, schema: str, key: List[int],
                             iv: List[int]):
        self.check_metadata()
        self.check_channel()
        request = RegisterOutputFileRequest(self.metadata, url,
                                            CryptoInfo(schema, key, iv))
        try:
            response = self.call_method(request)
            return response.data_id
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(
                f"Failed to register output file ({reason})")

    def register_input_from_output(self, data_id: str):
        """Register an input data from an output data.

        Args:

            data_id (str): ExternalID of the output data.

        Returns:

            str: ExternalID of input data
        """

        self.check_metadata()
        self.check_channel()
        request = RegisterInputFromOutputRequest(self.metadata, data_id)
        response = self.call_method(request)
        return response.data_id

    def register_fusion_output(self, owners: List[str] = []):
        """Register a fusion output data.

        Args:

            owners (List[OwnerList], optional): Owners of the output data. Defaults to [].
        
        Returns:

            str: ExternalID of fusion output data
        """

        request = RegisterFusionOutputRequest(self.metadata, owners)
        response = self.call_method(request)
        return response.data_id

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
        try:
            response = self.call_method(request)
            return response.task_id
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(f"Failed to create task ({reason})")

    def assign_data_to_task(self, task_id: str, inputs: List[DataMap],
                            outputs: List[DataMap]):
        self.check_metadata()
        self.check_channel()
        request = AssignDataRequest(self.metadata, task_id, inputs, outputs)
        try:
            self.call_method(request)
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(
                f"Failed to assign data to task ({reason})")

    def approve_task(self, task_id: str):
        self.check_metadata()
        self.check_channel()
        request = ApproveTaskRequest(self.metadata, task_id)
        try:
            self.call_method(request)
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(f"Failed to approve task ({reason})")

    def invoke_task(self, task_id: str):
        self.check_metadata()
        self.check_channel()
        request = InvokeTaskRequest(self.metadata, task_id)
        try:
            self.call_method(request)
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(f"Failed to invoke task ({reason})")

    def cancel_task(self, task_id: str):
        self.check_metadata()
        self.check_channel()
        request = CancelTaskRequest(self.metadata, task_id)
        try:
            self.call_method(request)
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(f"Failed to cancel task ({reason})")

    def get_task(self, task_id: str):
        self.check_metadata()
        self.check_channel()
        request = GetTaskRequest(self.metadata, task_id)
        try:
            response = self.call_method(request)
            return MessageToDict(response,
                                 preserving_proto_field_name=True,
                                 use_integers_for_enums=True)
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(f"Failed to get task result ({reason})")

    def get_task_result(self, task_id: str):
        self.check_metadata()
        self.check_channel()
        request = GetTaskRequest(self.metadata, task_id)
        while True:
            try:
                time.sleep(1)
                response = self.call_method(request)
                if response.status == TaskStatus.Finished:
                    break
                elif response.status == TaskStatus.Canceled:
                    raise TeaclaveException("Task Canceled, Error: " +
                                            response.result.Err.reason)
                elif response.status == TaskStatus.Failed:
                    raise TeaclaveException("Task Failed, Error: " +
                                            response.result.Err.reason)
            except Exception as e:
                reason = str(e)
                raise TeaclaveException(
                    f"Failed to get task result ({reason})")

        return response.result.Ok.return_value

    def get_output_cmac_by_tag(self, task_id: str, tag: str):
        self.check_metadata()
        self.check_channel()
        request = GetTaskRequest(self.metadata, task_id)
        while True:
            try:
                time.sleep(1)
                response = self.call_method(request)
                if response.status == TaskStatus.Finished:
                    break
            except Exception as e:
                reason = str(e)
                raise TeaclaveException(
                    f"Failed to get task result ({reason})")
        response = MessageToDict(response,
                                 preserving_proto_field_name=True,
                                 use_integers_for_enums=True)
        return base64.b64decode(response["result"]["Ok"]["tags_map"][tag])

    def query_audit_logs(self, message: str, limit: int):
        self.check_metadata()
        self.check_channel()
        request = QueryAuditLogsRequest(self.metadata, message, limit)
        try:
            response = self.call_method(request)
            return MessageToDict(response,
                                 preserving_proto_field_name=True,
                                 use_integers_for_enums=True)
        except Exception as e:
            reason = str(e)
            raise TeaclaveException(f"Failed to get audit logs ({reason})")
