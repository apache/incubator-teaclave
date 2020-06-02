#!/usr/bin/env python3

import sys
import base64

from teaclave import (AuthenticationService, FrontendService,
                      AuthenticationClient, FrontendClient)
from utils import (AUTHENTICATION_SERVICE_ADDRESS, FRONTEND_SERVICE_ADDRESS,
                   AS_ROOT_CA_CERT_PATH, ENCLAVE_INFO_PATH, USER_ID,
                   USER_PASSWORD)


class BuiltinOnlineDecryptExample:
    def __init__(self, user_id, user_password):
        self.user_id = user_id
        self.user_password = user_password

    def decrypt(self, key, nonce, encrypted_data, algorithm):
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
            name="builtin_online_decrypt",
            description="Native Echo Function",
            executor_type="builtin",
            arguments=["key", "nonce", "encrypted_data", "algorithm"])

        print("[+] creating task")
        task_id = client.create_task(
            function_id=function_id,
            function_arguments={
                "key": dataToBase64(key),
                "nonce": dataToBase64(nonce),
                "encrypted_data":
                "CaZd8qSMMlBp8SjSXj2I4dQIuC9KkZ5DI/ATo1sWJw==",
                "algorithm": algorithm
            },
            executor="builtin")

        print("[+] invoking task")
        client.invoke_task(task_id)

        print("[+] getting result")
        result = client.get_task_result(task_id)
        print("[+] done")

        return bytes(result)


def dataToBase64(data):
    return base64.standard_b64encode(bytes(data)).decode("utf-8")


def main():
    example = BuiltinOnlineDecryptExample(USER_ID, USER_PASSWORD)
    key = [
        106, 165, 29, 129, 157, 37, 38, 123, 179, 247, 40, 143, 146, 128, 241,
        51, 166, 92, 77, 197, 85, 165, 222, 10, 40, 186, 179, 108, 112, 252,
        240, 184
    ]
    nonce = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
    encrypted_data = "CaZd8qSMMlBp8SjSXj2I4dQIuC9KkZ5DI/ATo1sWJw=="
    algorithm = "aes256gcm"  # Alogorithm can be "aes256gcm" or "aes128gcm"
    rt = example.decrypt(key, nonce, encrypted_data, algorithm)
    print("[+] function return: ", rt)


if __name__ == '__main__':
    main()
