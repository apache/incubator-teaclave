#!/usr/bin/env python3

import socket
import ssl
import os
import time
import sys

from teaclave import read_message, write_message, verify_report

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
    payload = b"""
def entrypoint(argv):
    assert argv[0] == 'message'
    assert argv[1] is not None
    return argv[1]
"""
    message = {
        "metadata": {
            "id": user_id,
            "token": token
        },
        "request": "register_function",
        "name": "mesapy-echo",
        "description": "An echo function implemented in Python",
        "executor_type": "python",
        "public": True,
        "payload": list(payload),
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
        "executor": "mesapy",
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


class MesaPyEchoExample:
    def __init__(self, user_id, user_password):
        self.user_id = user_id
        self.user_password = user_password

    def echo(self, message="Hello, Teaclave!"):
        sock = socket.create_connection(AUTHENTICATION_SERVICE_ADDRESS)
        channel = CONTEXT.wrap_socket(sock, server_hostname=HOSTNAME)
        cert = channel.getpeercert(binary_form=True)
        verify_report(AS_ROOT_CA_CERT_PATH, ENCLAVE_INFO_PATH, cert, "authentication")

        print("[+] registering user")
        user_register(channel, self.user_id, self.user_password)

        print("[+] login")
        token = user_login(channel, self.user_id, self.user_password)

        sock = socket.create_connection(FRONTEND_SERVICE_ADDRESS)
        channel = CONTEXT.wrap_socket(sock, server_hostname=HOSTNAME)
        cert = channel.getpeercert(binary_form=True)
        verify_report(AS_ROOT_CA_CERT_PATH, ENCLAVE_INFO_PATH, cert, "frontend")

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
    example = MesaPyEchoExample(USER_ID, USER_PASSWORD)
    if len(sys.argv) > 1:
        message = sys.argv[1]
        rt = example.echo(message)
    else:
        rt = example.echo()

    print("[+] function return: ", rt)


if __name__ == '__main__':
    main()
