/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 *
 */

const char *authentication_service_address = "localhost:7776";
const char *frontend_service_address = "localhost:7777";
const char *enclave_info_path = "../../release/services/enclave_info.toml";
#ifdef DCAP
const char *as_root_ca_cert_path = "../../keys/dcap_root_ca_cert.pem";
#else
const char *as_root_ca_cert_path = "../../keys/ias_root_ca_cert.pem";
#endif

int user_register(const char* admin_user_id,
             const char* token,
             const char* user_id,
             const char* user_password) {
    int ret = 0;

    AuthenticationClient *authentication_client =
        teaclave_connect_authentication_service(authentication_service_address,
                                                enclave_info_path,
                                                as_root_ca_cert_path);
    if (authentication_client == NULL) {
        fprintf(stderr,
                "[-] Failed to connect to the authentication service.\n");
        ret = 1;
        goto bail;
    }

    ret = teaclave_authentication_set_credential(authentication_client, admin_user_id, token);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to login.\n");
        goto bail;
    }

    ret = teaclave_user_register(authentication_client, user_id, user_password, "PlatformAdmin", "");
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to register user.\n");
        fprintf(stderr, "[-] Maybe `%s' already exists. Continue. \n", user_id);
    }
bail:
    if (authentication_client) {
        int teaclave_close_rv =
            teaclave_close_authentication_service(authentication_client);
        if (teaclave_close_rv != 0) {
            fprintf(stderr,
                    "[-] Failed to close the authentication service client.\n");
        }
    }

    return ret;
}

int login(const char* user_id, const char* user_password, char *token, size_t *token_len) {
    int ret = 0;

    AuthenticationClient *authentication_client =
        teaclave_connect_authentication_service(authentication_service_address,
                                                enclave_info_path,
                                                as_root_ca_cert_path);
    if (authentication_client == NULL) {
        fprintf(stderr,
                "[-] Failed to connect to the authentication service.\n");
        ret = 1;
        goto bail;
    }

    ret = teaclave_user_login(authentication_client, user_id, user_password,
                              token, token_len);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to login.\n");
        goto bail;
    }
    printf("[+] token: %s\n", token);

bail:
    if (authentication_client) {
        int teaclave_close_rv =
            teaclave_close_authentication_service(authentication_client);
        if (teaclave_close_rv != 0) {
            fprintf(stderr,
                    "[-] Failed to close the authentication service client.\n");
        }
    }

    return ret;
}
