#!/usr/bin/env python3

import socket
import struct
import ssl
import json
import base64
import toml
import os
import time
import sys

from cryptography import x509
from cryptography.hazmat.backends import default_backend

from OpenSSL.crypto import load_certificate, FILETYPE_PEM, FILETYPE_ASN1
from OpenSSL.crypto import X509Store, X509StoreContext
from OpenSSL import crypto

HOSTNAME = 'localhost'
AUTHENTICATION_SERVICE_ADDRESS = (HOSTNAME, 7776)
FRONTEND_SERVICE_ADDRESS = (HOSTNAME, 7777)
CONTEXT = ssl._create_unverified_context()

USER_ID = "example_user"
USER_PASSWORD = "test_password"

if os.environ.get('DCAP'):
    AS_ROOT_CERT_FILENAME = "dcap_root_ca_cert.pem"
else:
    AS_ROOT_CERT_FILENAME = "ias_root_ca_cert.pem"

if os.environ.get('TEACLAVE_PROJECT_ROOT'):
    AS_ROOT_CA_CERT_PATH = os.environ['TEACLAVE_PROJECT_ROOT'] + \
        "/keys/" + AS_ROOT_CERT_FILENAME
    ENCLAVE_INFO_PATH = os.environ['TEACLAVE_PROJECT_ROOT'] + \
        "/release/tests/enclave_info.toml"
else:
    AS_ROOT_CA_CERT_PATH = "../../keys/" + AS_ROOT_CERT_FILENAME
    ENCLAVE_INFO_PATH = "../../release/tests/enclave_info.toml"


def write_message(sock, message):
    message = json.dumps(message)
    message = message.encode()
    sock.write(struct.pack(">Q", len(message)))
    sock.write(message)


def read_message(sock):
    response_len = struct.unpack(">Q", sock.read(8))
    response = sock.read(response_len[0])
    response = json.loads(response)
    return response


def verify_report(cert, endpoint_name):
    if os.environ.get('SGX_MODE') == 'SW':
        return

    cert = x509.load_der_x509_certificate(cert, default_backend())
    ext = json.loads(cert.extensions[0].value.value)

    report = bytes(ext["report"])
    signature = bytes(ext["signature"])
    signing_cert = bytes(ext["signing_cert"])
    signing_cert = load_certificate(FILETYPE_ASN1, signing_cert)

    # verify signing cert with AS root cert
    with open(AS_ROOT_CA_CERT_PATH) as f:
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
    mr_enclave = quote[112:112+32].hex()
    mr_signer = quote[176:176+32].hex()

    # get enclave_info
    enclave_info = toml.load(ENCLAVE_INFO_PATH)

    # verify mr_enclave and mr_signer
    enclave_name = "teaclave_" + endpoint_name + "_service"
    if mr_enclave != enclave_info[enclave_name]["mr_enclave"]:
        raise Exception("mr_enclave error")

    if mr_signer != enclave_info[enclave_name]["mr_signer"]:
        raise Exception("mr_signer error")


def user_register(channel, user_id, user_password):
    message = {
        "request": "user_register",
        "id": user_id,
        "password": user_password
    }
    write_message(channel, message)
    read_message(channel)


def user_login(channel, user_id, user_password):
    message = {
        "request": "user_login",
        "id": user_id,
        "password": user_password
    }
    write_message(channel, message)

    response = read_message(channel)
    assert(response["result"] == "ok")
    return response["content"]["token"]


def register_function(channel, user_id, token):
    message = {
        "metadata": {
            "id": user_id,
            "token": token
        },
        "request": "register_function",
        "name": "builtin-echo",
        "description": "Native Echo Function",
        "executor_type": "builtin",
        "public": True,
        "payload": [],
        "arguments": ["message"],
        "inputs": [],
        "outputs": [],
    }
    write_message(channel, message)

    response = read_message(channel)
    return response["content"]["function_id"]


def create_task(channel, user_id, token, function_id, message):
    message = {
        "metadata": {
            "id": user_id,
            "token": token
        },
        "request": "create_task",
        "function_id": function_id,
        "function_arguments": {
            "message": message,
        },
        "executor": "builtin",
        "inputs_ownership": [],
        "outputs_ownership": [],
    }
    write_message(channel, message)

    response = read_message(channel)
    return response["content"]["task_id"]


def approve_task(channel, user_id, token, task_id):
    message = {
        "metadata": {
            "id": user_id,
            "token": token
        },
        "request": "approve_task",
        "task_id": task_id,
    }
    write_message(channel, message)

    response = read_message(channel)
    assert(response["result"] == "ok")


def invoke_task(channel, user_id, token, task_id):
    message = {
        "metadata": {
            "id": user_id,
            "token": token
        },
        "request": "invoke_task",
        "task_id": task_id,
    }
    write_message(channel, message)

    response = read_message(channel)
    assert(response["result"] == "ok")


def get_task_result(channel, user_id, token, task_id):
    message = {
        "metadata": {
            "id": user_id,
            "token": token
        },
        "request": "get_task",
        "task_id": task_id,
    }
    while True:
        write_message(channel, message)
        response = read_message(channel)
        time.sleep(1)
        if response["content"]["status"] == 10:
            break

    return response["content"]["result"]["result"]["Ok"]["return_value"]


class BuiltinEchoExample:
    def __init__(self, user_id, user_password):
        self.user_id = user_id
        self.user_password = user_password

    def echo(self, message="Hello, Teaclave!"):
        sock = socket.create_connection(AUTHENTICATION_SERVICE_ADDRESS)
        channel = CONTEXT.wrap_socket(sock, server_hostname=HOSTNAME)
        cert = channel.getpeercert(binary_form=True)
        verify_report(cert, "authentication")

        print("[+] registering user")
        user_register(channel, self.user_id, self.user_password)

        print("[+] login")
        token = user_login(channel, self.user_id, self.user_password)

        sock = socket.create_connection(FRONTEND_SERVICE_ADDRESS)
        channel = CONTEXT.wrap_socket(sock, server_hostname=HOSTNAME)
        cert = channel.getpeercert(binary_form=True)
        verify_report(cert, "frontend")

        print("[+] registering function")
        function_id = register_function(channel, self.user_id, token)

        print("[+] creating task")
        task_id = create_task(channel, self.user_id,
                              token, function_id, message)
        print("[+] approving task")
        approve_task(channel, self.user_id, token, task_id)

        print("[+] invoking task")
        invoke_task(channel, self.user_id, token, task_id)

        print("[+] getting result")
        result = get_task_result(channel, self.user_id, token, task_id)
        print("[+] done")

        return bytes(result)


def main():
    example = BuiltinEchoExample(USER_ID, USER_PASSWORD)
    if len(sys.argv) > 1:
        message = sys.argv[1]
        rt = example.echo(message)
    else:
        rt = example.echo()

    print("[+] function return: ", rt)


if __name__ == '__main__':
    main()
