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

import os
from teaclave import (AuthenticationService, FrontendService)

HOSTNAME = 'localhost'
AUTHENTICATION_SERVICE_ADDRESS = (HOSTNAME, 7776)
FRONTEND_SERVICE_ADDRESS = (HOSTNAME, 7777)

USER_ID = "admin"
USER_PASSWORD = "teaclave"

if os.environ.get('DCAP'):
    AS_ROOT_CERT_FILENAME = "dcap_root_ca_cert.pem"
else:
    AS_ROOT_CERT_FILENAME = "ias_root_ca_cert.pem"

if os.environ.get('TEACLAVE_PROJECT_ROOT'):
    AS_ROOT_CA_CERT_PATH = os.environ['TEACLAVE_PROJECT_ROOT'] + \
        "/config/keys/" + AS_ROOT_CERT_FILENAME
    ENCLAVE_INFO_PATH = os.environ['TEACLAVE_PROJECT_ROOT'] + \
        "/release/tests/enclave_info.toml"
else:
    AS_ROOT_CA_CERT_PATH = "../../config/keys/" + AS_ROOT_CERT_FILENAME
    ENCLAVE_INFO_PATH = "../../release/examples/enclave_info.toml"


class PlatformAdmin:

    def __init__(self, user_id: str, user_password: str):
        self.client = AuthenticationService(AUTHENTICATION_SERVICE_ADDRESS,
                                            AS_ROOT_CA_CERT_PATH,
                                            ENCLAVE_INFO_PATH)
        token = self.client.user_login(user_id, user_password)
        self.client.metadata = {"id": user_id, "token": token}

    def register_user(self, user_id: str, user_password: str):
        self.client.user_register(user_id, user_password, "PlatformAdmin", "")


def connect_authentication_service():
    return AuthenticationService(AUTHENTICATION_SERVICE_ADDRESS,
                                 AS_ROOT_CA_CERT_PATH, ENCLAVE_INFO_PATH)


def connect_frontend_service():
    return FrontendService(FRONTEND_SERVICE_ADDRESS, AS_ROOT_CA_CERT_PATH,
                           ENCLAVE_INFO_PATH)
