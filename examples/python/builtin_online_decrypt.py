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


class BuiltinEchoExample:
    def __init__(self, user_id, user_password):
        self.user_id = user_id
        self.user_password = user_password

    def echo(self, key_file_id="Hello, Teaclave!"):
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
            name="builtin-online-decrypt",
            description="Native Echo Function",
            executor_type="builtin",
            arguments=["key", "nonce", "encrypted_data"])

        print("[+] creating task")
        task_id = client.create_task(function_id=function_id,
                                     function_arguments={"key": "aqUdgZ0lJnuz9yiPkoDxM6ZcTcVVpd4KKLqzbHD88Lg=",
                                                         "nonce": "AAECAwQFBgcICQoL",
                                                         "encrypted_data": "CaZd8qSMMlBp8SjSXj2I4dQIuC9KkZ5DI/ATo1sWJw=="},
                                     executor="builtin")

        print("[+] invoking task")
        client.invoke_task(task_id)

        print("[+] getting result")
        result = client.get_task_result(task_id)
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
