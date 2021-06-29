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

#include "teaclave_client_sdk.h"
#include <stdio.h>
#include <string.h>

#define BUFFER_SIZE 4086
#define QUOTE(x...) #x

const char *authentication_service_address = "localhost:7776";
const char *frontend_service_address = "localhost:7777";
const char *enclave_info_path = "../../release/services/enclave_info.toml";
#ifdef DCAP
const char *as_root_ca_cert_path = "../../keys/dcap_root_ca_cert.pem";
#else
const char *as_root_ca_cert_path = "../../keys/ias_root_ca_cert.pem";
#endif
const char *user_id = "test_id";
const char *user_password = "test_password";

const char *register_function_request_serialized = QUOTE({
    "request" : "register_function",
    "name" : "builtin-echo",
    "description" : "Native Echo Function",
    "executor_type" : "builtin",
    "public" : true,
    "payload" : [],
    "arguments" : ["message"],
    "inputs" : [],
    "outputs" : []
});

const char *create_task_request_serialized = QUOTE({
    "request" : "create_task",
    "function_id" : "%s",
    "function_arguments" : "{\"message\": \"Hello, Teaclave!\"}",
    "executor" : "builtin",
    "inputs_ownership" : [],
    "outputs_ownership" : []
});

int login(char *token, size_t *token_len) {
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

    ret = teaclave_user_register(authentication_client, user_id, user_password);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to register user.\n");
        fprintf(stderr, "[-] Maybe `%s' already exists. Continue. \n", user_id);
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

int main() {
    int ret = 0;

    char token[BUFFER_SIZE] = {0};
    char serialized_response[BUFFER_SIZE] = {0};
    char function_id[BUFFER_SIZE] = {0};
    char serialized_request[BUFFER_SIZE] = {0};
    char task_result[BUFFER_SIZE] = {0};
    char task_id[BUFFER_SIZE] = {0};

    /* Login. */
    size_t token_len = BUFFER_SIZE;
    ret = login(token, &token_len);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to login.\n");
        goto bail;
    }

    /* Connect to the frontend serivice. */
    FrontendClient *frontend_client = teaclave_connect_frontend_service(
        frontend_service_address, enclave_info_path, as_root_ca_cert_path);
    if (frontend_client == NULL) {
        fprintf(stderr, "[-] Failed to connect to the frontend service.\n");
        ret = 1;
        goto bail;
    }

    /* Set user id and token. */
    ret = teaclave_set_credential(frontend_client, user_id, token);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to set credential.\n");
        goto bail;
    }

    /* Register function. */
    size_t serialized_response_len = BUFFER_SIZE;
    ret = teaclave_register_function_serialized(
        frontend_client, register_function_request_serialized,
        serialized_response, &serialized_response_len);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to register the function.\n");
        goto bail;
    }

    sscanf(serialized_response, "{\"function_id\":\"%45s", function_id);
    printf("[+] function_id: %s\n", function_id);

    /* Create task. */
    snprintf(serialized_request, BUFFER_SIZE, create_task_request_serialized,
             function_id);

    memset(serialized_response, 0, BUFFER_SIZE);
    ret = teaclave_create_task_serialized(frontend_client, serialized_request,
                                          serialized_response,
                                          &serialized_response_len);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to create a task.\n");
        goto bail;
    }

    sscanf(serialized_response, "{\"task_id\":\"%41s", task_id);
    printf("[+] task_id: %s\n", task_id);

    /* Invoke task. */
    ret = teaclave_invoke_task(frontend_client, task_id);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to invoke the task.\n");
        goto bail;
    }

    /* Get task result. */
    size_t task_result_len = BUFFER_SIZE;
    ret = teaclave_get_task_result(frontend_client, task_id, task_result,
                                   &task_result_len);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to get the task result.\n");
        goto bail;
    }

    printf("[+] Task result in string: %s\n", task_result);

    ret = teaclave_close_frontend_service(frontend_client);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to close the frontend service client.\n");
    }

    return ret;

bail:
    ret = teaclave_close_frontend_service(frontend_client);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to close the frontend service client.\n");
    }

    exit(-1);
}
