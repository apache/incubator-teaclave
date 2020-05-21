#!/usr/bin/env python3

import struct
import json
import base64
import toml
import os
import time
import ssl
import socket

from cryptography import x509
from cryptography.hazmat.backends import default_backend

from OpenSSL.crypto import load_certificate, FILETYPE_PEM, FILETYPE_ASN1
from OpenSSL.crypto import X509Store, X509StoreContext
from OpenSSL import crypto


class RequestEncoder(json.JSONEncoder):
    def default(self, o):
        return o.__dict__


class UserRegisterReqeust:
    def __init__(self, user_id, user_password):
        self.request = "user_register"
        self.id = user_id
        self.password = user_password


class UserLoginRequest:
    def __init__(self, user_id, user_password):
        self.request = "user_login"
        self.id = user_id
        self.password = user_password


class AuthenticationClient:
    def __init__(self, channel):
        self.channel = channel

    def user_register(self, user_id, user_password):
        request = UserRegisterReqeust(user_id, user_password)
        write_message(self.channel, request)
        return read_message(self.channel)

    def user_login(self, user_id, user_password):
        request = UserLoginRequest(user_id, user_password)
        write_message(self.channel, request)
        response = read_message(self.channel)
        return response["content"]["token"]


class AuthenticationService:
    context = ssl._create_unverified_context()

    def __init__(self, address, as_root_ca_cert_path, enclave_info_path):
        self.address = address
        self.as_root_ca_cert_path = as_root_ca_cert_path
        self.enclave_info_path = enclave_info_path

    def connect(self):
        sock = socket.create_connection(self.address)
        channel = self.context.wrap_socket(sock,
                                           server_hostname=self.address[0])
        cert = channel.getpeercert(binary_form=True)
        verify_report(self.as_root_ca_cert_path, self.enclave_info_path, cert,
                      "authentication")

        return channel


class FrontendService:
    context = ssl._create_unverified_context()

    def __init__(self, address, as_root_ca_cert_path, enclave_info_path):
        self.address = address
        self.as_root_ca_cert_path = as_root_ca_cert_path
        self.enclave_info_path = enclave_info_path

    def connect(self):
        sock = socket.create_connection(self.address)
        channel = self.context.wrap_socket(sock,
                                           server_hostname=self.address[0])
        cert = channel.getpeercert(binary_form=True)
        verify_report(self.as_root_ca_cert_path, self.enclave_info_path, cert,
                      "frontend")

        return channel


class RegisterFunctionRequest:
    def __init__(self, metadata, name, description, executor_type, public,
                 payload, arguments, inputs, outputs):
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


class CreateTaskRequest:
    def __init__(self, metadata, function_id, function_arguments, executor,
                 inputs_ownership, outputs_ownership):
        self.request = "create_task"
        self.metadata = metadata
        self.function_id = function_id
        self.function_arguments = function_arguments
        self.executor = executor
        self.inputs_ownership = inputs_ownership
        self.outputs_ownership = outputs_ownership


class InvokeTaskRequest:
    def __init__(self, metadata, task_id):
        self.request = "invoke_task"
        self.metadata = metadata
        self.task_id = task_id


class GetTaskRequest:
    def __init__(self, metadata, task_id):
        self.request = "get_task"
        self.metadata = metadata
        self.task_id = task_id


class FrontendClient:
    def __init__(self, channel, metadata):
        self.channel = channel
        self.metadata = metadata

    def register_function(self,
                          name,
                          description,
                          executor_type,
                          public=True,
                          payload=[],
                          arguments=[],
                          inputs=[],
                          outputs=[]):
        request = RegisterFunctionRequest(self.metadata, name, description,
                                          executor_type, public, payload,
                                          arguments, inputs, outputs)
        write_message(self.channel, request)
        response = read_message(self.channel)
        return response["content"]["function_id"]

    def create_task(self,
                    function_id,
                    function_arguments,
                    executor,
                    inputs_ownership=[],
                    outputs_ownership=[]):
        function_arguments = json.dumps(function_arguments)
        request = CreateTaskRequest(self.metadata, function_id,
                                    function_arguments, executor,
                                    inputs_ownership, outputs_ownership)
        write_message(self.channel, request)
        response = read_message(self.channel)
        return response["content"]["task_id"]

    def invoke_task(self, task_id):
        request = InvokeTaskRequest(self.metadata, task_id)
        write_message(self.channel, request)
        response = read_message(self.channel)
        assert (response["result"] == "ok")

    def get_task_result(self, task_id):
        request = GetTaskRequest(self.metadata, task_id)

        while True:
            write_message(self.channel, request)
            response = read_message(self.channel)
            time.sleep(1)
            if response["content"]["status"] == 10:
                break

        return response["content"]["result"]["result"]["Ok"]["return_value"]


def write_message(sock, message):
    message = json.dumps(message, cls=RequestEncoder).encode()
    sock.write(struct.pack(">Q", len(message)))
    sock.write(message)


def read_message(sock):
    response_len = struct.unpack(">Q", sock.read(8))
    response = sock.read(response_len[0])
    response = json.loads(response)
    return response


def verify_report(as_root_ca_cert_path, enclave_info_path, cert,
                  endpoint_name):
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
