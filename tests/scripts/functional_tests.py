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

import unittest
import socket
import struct
import ssl
import json
import base64
import toml
import os

from cryptography import x509
from cryptography.hazmat.backends import default_backend

from OpenSSL.crypto import load_certificate, FILETYPE_PEM, FILETYPE_ASN1
from OpenSSL.crypto import X509Store, X509StoreContext
from OpenSSL import crypto

import h2.connection
import h2.events

from io import BytesIO
from h2.config import H2Configuration
from urllib.parse import unquote
from teaclave_authentication_service_pb2 import UserLoginRequest, UserLoginResponse

HOSTNAME = 'localhost'
AUTHENTICATION_SERVICE_ADDRESS = (HOSTNAME, 7776)
CONTEXT = ssl._create_unverified_context()

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


def verify_report(cert, endpoint_name):

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
    ext = json.loads(cert.extensions[0].value.value)

    report = bytes(ext["report"])
    signature = bytes(ext["signature"])
    certs = [load_certificate(FILETYPE_ASN1, bytes(c)) for c in ext["certs"]]

    # verify signing cert with AS root cert
    with open(AS_ROOT_CA_CERT_PATH) as f:
        as_root_ca_cert = f.read()
    as_root_ca_cert = load_certificate(FILETYPE_PEM, as_root_ca_cert)
    store = X509Store()
    store.add_cert(as_root_ca_cert)
    for c in certs:
        store.add_cert(c)
    store_ctx = X509StoreContext(store, as_root_ca_cert)
    store_ctx.verify_certificate()

    # verify report's signature
    crypto.verify(certs[0], signature, bytes(ext["report"]), 'sha256')

    report = json.loads(report)
    quote = report['isvEnclaveQuoteBody']
    quote = base64.b64decode(quote)

    # get mr_enclave and mr_signer from the quote
    mr_enclave = quote[112:112 + 32].hex()
    mr_signer = quote[176:176 + 32].hex()

    # get enclave_info
    enclave_info = toml.load(ENCLAVE_INFO_PATH)

    # verify mr_enclave and mr_signer
    enclave_name = "teaclave_" + endpoint_name + "_service"
    if mr_enclave != enclave_info[enclave_name]["mr_enclave"]:
        raise Exception("mr_enclave error")

    if mr_signer != enclave_info[enclave_name]["mr_signer"]:
        raise Exception("mr_signer error")


def encode_message(message):
    message_bin = message.SerializeToString()
    header = struct.pack('?', False) + struct.pack('>I', len(message_bin))
    return header + message_bin


def decode_message(message_bin, message_type):
    f = BytesIO(message_bin)
    meta = f.read(5)
    message_len = struct.unpack('>I', meta[1:])[0]
    message_body = f.read(message_len)
    message = message_type.FromString(message_body)
    return message


class TestAuthenticationService(unittest.TestCase):

    def setUp(self):
        sock = socket.create_connection(AUTHENTICATION_SERVICE_ADDRESS)
        CONTEXT.set_alpn_protocols(['h2'])
        self.socket = CONTEXT.wrap_socket(sock, server_hostname=HOSTNAME)
        cert = self.socket.getpeercert(binary_form=True)
        verify_report(cert, "authentication")
        config = H2Configuration(client_side=True, header_encoding='ascii')
        self.connection = h2.connection.H2Connection(config)
        self.connection.initiate_connection()
        self.socket.sendall(self.connection.data_to_send())
        self.stream_id = 1

    def set_headers(self, method_path):
        headers = [(':method', 'POST'), (':path', method_path),
                   (':authority', HOSTNAME), (':scheme', 'https'),
                   ('content-type', 'application/grpc')]
        return headers

    def send_message(self, message, method_path):
        headers = self.set_headers(method_path)
        self.connection.send_headers(self.stream_id, headers)
        message_data = encode_message(message)
        self.connection.send_data(self.stream_id,
                                  message_data,
                                  end_stream=True)
        self.socket.sendall(self.connection.data_to_send())

    def recv_message(self):
        body = None
        headers = None
        response_stream_ended = False
        max_frame_size = self.connection.max_outbound_frame_size
        print(max_frame_size)
        while not response_stream_ended:
            # read raw data from the socket
            data = self.socket.recv(max_frame_size)
            if not data:
                break

            # feed raw data into h2, and process resulting events
            events = self.connection.receive_data(data)
            for event in events:
                if isinstance(event, h2.events.ResponseReceived):
                    headers = dict(event.headers)
                if isinstance(event, h2.events.DataReceived):
                    # update flow control so the server doesn't starve us
                    self.connection.acknowledge_received_data(
                        event.flow_controlled_length, event.stream_id)
                    # more response body data received
                    body += event.data
                if isinstance(event, h2.events.StreamEnded):
                    # response body completed, let's exit the loop
                    response_stream_ended = True
                    break
            # send any pending data to the server
            self.socket.sendall(self.connection.data_to_send())
        return (headers, body)

    def tearDown(self):
        self.connection.close_connection()
        self.socket.sendall(self.connection.data_to_send())
        self.socket.close()

    def test_invalid_request(self):
        path = '/teaclave_authentication_service_proto.TeaclaveAuthenticationApi/InvalidRequest'
        user_id = "invalid_id"
        user_password = "invalid_password"

        message = UserLoginRequest(id=user_id, password=user_password)
        self.send_message(message, path)

        (headers, response) = self.recv_message()
        self.assertEqual(response, None)
        # https://grpc.github.io/grpc/core/md_doc_statuscodes.html
        # grpc status UNIMPLEMENTED: 12
        self.assertEqual(headers['grpc-status'], '12')

    def test_login_permission_denied(self):
        path = '/teaclave_authentication_service_proto.TeaclaveAuthenticationApi/UserLogin'
        user_id = "invalid_id"
        user_password = "invalid_password"

        message = UserLoginRequest(id=user_id, password=user_password)
        self.send_message(message, path)
        (headers, body) = self.recv_message()
        self.assertEqual(body, None)
        self.assertEqual(headers['grpc-status'], '16')
        message = unquote(headers['grpc-message'],
                          encoding='utf-8',
                          errors='replace')
        self.assertEqual(message, 'authentication failed')


if __name__ == '__main__':
    unittest.main()
