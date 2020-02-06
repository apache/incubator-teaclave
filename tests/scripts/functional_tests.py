#!/usr/bin/env python3

import unittest
import socket
import struct
import ssl
import json

hostname = 'localhost'
authentication_service_address = (hostname, 7776)
context = ssl._create_unverified_context()


def write_message(sock, message):
    message = json.dumps(message)
    message = message.encode()
    sock.write(struct.pack(">Q", len(message)))
    sock.write(message)


def read_message(sock):
    response_len = struct.unpack(">Q", sock.read(8))
    response = sock.read(response_len[0])
    return response


class TestAuthenticationService(unittest.TestCase):

    def setUp(self):
        sock = socket.create_connection(authentication_service_address)
        self.socket = context.wrap_socket(sock, server_hostname=hostname)

    def tearDown(self):
        self.socket.close()

    def test_invalid_request(self):
        user_id = "invalid_id"
        user_password = "invalid_password"

        message = {
            "invalid_request": "user_login",
            "id": user_id,
            "password": user_password
        }
        write_message(self.socket, message)

        response = read_message(self.socket)
        self.assertEqual(
            response, b'{"result":"err","request_error":"invalid request"}')

    def test_login_permission_denied(self):
        user_id = "invalid_id"
        user_password = "invalid_password"

        message = {
            "request": "user_login",
            "id": user_id,
            "password": user_password
        }
        write_message(self.socket, message)

        response = read_message(self.socket)
        self.assertEqual(
            response, b'{"result":"err","request_error":"permission denied"}')


if __name__ == '__main__':
    unittest.main()
