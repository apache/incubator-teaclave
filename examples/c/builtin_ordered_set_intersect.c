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

#include "utils.h"

#define BUFFER_SIZE 4086
#define QUOTE(x...) #x

typedef struct UserData {
    char *user_id;
    char *password;
    char *input_url;
    char *output_url;
    char input_cmac[16];
    char key[16];
} UserData;

struct UserData user0_data = {
    .user_id = "user0",
    .password = "password",
    .input_url = "http://localhost:6789/fixtures/functions/ordered_set_intersect/psi0.txt.enc",
    .output_url = "http://localhost:6789/fixtures/functions/ordered_set_intersect/output_psi0.enc",
    .input_cmac = {0x92, 0xf6, 0x86, 0xd4, 0xac, 0x2b, 0xa6, 0xb4, 0xff, 0xd9, 0x3b, 0xc7, 0xac, 0x5d, 0xbf, 0x58},
    .key = {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}};

struct UserData user1_data = {
    .user_id = "user1",
    .password = "password",
    .input_url = "http://localhost:6789/fixtures/functions/ordered_set_intersect/psi1.txt.enc",
    .output_url = "http://localhost:6789/fixtures/functions/ordered_set_intersect/output_psi1.enc",
    .input_cmac = {0x8b, 0x31, 0x04, 0x97, 0x2a, 0x6f, 0x0d, 0xe9, 0x49, 0x31, 0x5e, 0x0b, 0x45, 0xd5, 0xdd, 0x66},
    .key = {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}};

const char *register_function_request_serialized = QUOTE(
{
    "request": "register_function",
    "name": "builtin-ordered-set-intersect",
    "description": "Native Private Set Intersection",
    "executor_type": "builtin",
    "public": true,
    "payload": [],
    "arguments": [{"key": "order", "default_value": "", "allow_overwrite": true}],
    "inputs": [
        {"name": "input_data1", "description": "Client 0 data.", "optional": false},
        {"name": "input_data2", "description": "Client 1 data.", "optional": false}
    ],
    "outputs": [ 
        {"name": "output_result1", "description": "Output data.", "optional": false},
        {"name": "output_result2", "description": "Output data.", "optional": false}
    ],
    "user_allowlist": ["user0", "user1"],
    "usage_quota": -1
});

const char *create_task_request_serialized = QUOTE(
{
    "request": "create_task",
    "function_id": "%s",
    "function_arguments": "{\"order\": \"ascending\"}",
    "executor": "builtin",
    "inputs_ownership": [
        {"data_name": "input_data1", "uids": ["user0"]},
        {"data_name": "input_data2", "uids": ["user1"]}
    ],
    "outputs_ownership": [
        {"data_name": "output_result1", "uids": ["user0"]},
        {"data_name": "output_result2", "uids": ["user1"]}
    ]
});

const char *register_input_serialized= QUOTE(
{
    "request": "register_input_file",
    "url": "%s",
    "cmac": %s,
    "crypto_info": {
        "schema": "teaclave-file-128",
        "key": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        "iv": []
    }
});

const char *register_output_serialized= QUOTE(
{
    "request": "register_output_file",
    "url": "%s",
    "crypto_info": {
        "schema": "teaclave-file-128",
        "key": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        "iv": []
    }
});

const char *user0_assign_serialized= QUOTE(
{
    "request": "assign_data",
    "task_id": "%s",
    "inputs": [
        {"data_name": "input_data1", "data_id": "%s"}
    ],
    "outputs": [
        {"data_name": "output_result1", "data_id": "%s"}
    ]
});

const char *user1_assign_serialized= QUOTE(
{
    "request": "assign_data",
    "task_id": "%s",
    "inputs": [
        {"data_name": "input_data2", "data_id": "%s"}
    ],
    "outputs": [
        {"data_name": "output_result2", "data_id": "%s"}
    ]
});

const char *approve_serialized = QUOTE(
{
    "request": "approve_task",
    "task_id": "%s"
});

struct FrontendClient *init_client(char *user_id, char *password)
{
    struct FrontendClient *frontend_client;
    int ret;
    char token[BUFFER_SIZE] = {0};
    size_t token_len = BUFFER_SIZE;

    /* Login. */
    ret = login(user_id, password, token, &token_len);
    if (ret != 0) {
        fprintf(stderr, "[-] %s Failed to login.\n", user_id);
    }

    /* Connect to the frontend serivice. */
    frontend_client =
        teaclave_connect_frontend_service(frontend_service_address, enclave_info_path, as_root_ca_cert_path);
    if (frontend_client == NULL) {
        fprintf(stderr, "[-] %s Failed to connect to the frontend service.\n", user_id);
        return frontend_client;
    }

    /* Set user id and token. */
    ret = teaclave_frontend_set_credential(frontend_client, user_id, token);
    if (ret != 0) {
        fprintf(stderr, "[-] %s Failed to set credential.\n", user_id);
        return frontend_client;
    }

    return frontend_client;
}

int set_task(struct FrontendClient *frontend_client, char *serialized_response, size_t serialized_response_len)
{
    /* Register function. */
    int ret = 0;
    char serialized_request[BUFFER_SIZE] = {0};
    char function_id[BUFFER_SIZE] = {0};
    memset(serialized_response, 0, BUFFER_SIZE);
    serialized_response_len = BUFFER_SIZE;
    ret = teaclave_register_function_serialized(frontend_client, register_function_request_serialized,
                                                serialized_response, &serialized_response_len);

    if (ret != 0) {
        fprintf(stderr, "[-] Failed to register the function.\n");
        return ret;
    }
    sscanf(serialized_response, "{\"function_id\":\"%45s", function_id);
    printf("[+] function_id: %s\n", function_id);

    /* Create task. */
    snprintf(serialized_request, BUFFER_SIZE, create_task_request_serialized, function_id);
    memset(serialized_response, 0, BUFFER_SIZE);
    serialized_response_len = BUFFER_SIZE;
    ret = teaclave_create_task_serialized(frontend_client, serialized_request, serialized_response,
                                          &serialized_response_len);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to create a task.\n");
    }
    return ret;
}

char *format_cmac_to_string(UserData user_data){
    static char cmac[128] = {0};
    int n = 0;
    n += sprintf (&cmac[n], "[%hhu", user_data.input_cmac[0]);
    for (int i = 1; i < 16; i++) {
        n += sprintf (&cmac[n], ",%hhu", user_data.input_cmac[i]);
    }
    sprintf (&cmac[n], "]");
    return cmac;
}

int main()
{
    int ret = 0;
    char token[BUFFER_SIZE] = {0};
    size_t token_len = BUFFER_SIZE;
    char task_id[BUFFER_SIZE] = {0};
    char user0_task_result[BUFFER_SIZE] = {0};
    char user1_task_result[BUFFER_SIZE] = {0};
    char user0_serialized_input_request[BUFFER_SIZE] = {0};
    char user0_serialized_output_request[BUFFER_SIZE] = {0};
    char user0_serialized_assign_request[BUFFER_SIZE] = {0};
    char user1_serialized_input_request[BUFFER_SIZE] = {0};
    char user1_serialized_output_request[BUFFER_SIZE] = {0};
    char user1_serialized_assign_request[BUFFER_SIZE] = {0};
    char user0_approve_request[BUFFER_SIZE] = {0};
    char user1_approve_request[BUFFER_SIZE] = {0};
    char user0_input_id[BUFFER_SIZE] = {0};
    char user0_output_id[BUFFER_SIZE] = {0};
    char user1_input_id[BUFFER_SIZE] = {0};
    char user1_output_id[BUFFER_SIZE] = {0};
    char serialized_response[BUFFER_SIZE] = {0};
    size_t serialized_response_len = BUFFER_SIZE;
    const char *admin_user_id = "admin";
    const char *admin_user_password = "teaclave";

    /* Register */
    ret = login(admin_user_id, admin_user_password, token, &token_len);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to login.\n");
        goto bail;
    }

    ret = user_register(admin_user_id, token, "user0", "password");
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to login. Ignored.\n");
    }

    ret = user_register(admin_user_id, token, "user1", "password");
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to login. Ignored.\n");
    }

    FrontendClient *client0 = init_client(user0_data.user_id, user0_data.password);
    if (client0 == NULL) {
        fprintf(stderr, "[-] %s Failed to init the client.\n", user0_data.user_id);
        goto bail;
    }

    FrontendClient *client1 = init_client(user1_data.user_id, user1_data.password);
    if (client0 == NULL) {
        fprintf(stderr, "[-] %s Failed to init the client.\n", user1_data.user_id);
        goto bail;
    }

    ret = set_task(client0, serialized_response, serialized_response_len);
    if (ret != 0) {
        fprintf(stderr, "[-] %s failed to set the task.\n", user0_data.user_id);
        goto bail;
    }
    sscanf(serialized_response, "{\"task_id\":\"%41s", task_id);
    printf("[+] task_id: %s\n", task_id);

    /* User0 register input data. */
    printf("[+] %s register input data\n", user0_data.user_id);
    snprintf(user0_serialized_input_request, BUFFER_SIZE, register_input_serialized, user0_data.input_url,
         format_cmac_to_string(user0_data));
    memset(serialized_response, 0, BUFFER_SIZE);
    serialized_response_len = BUFFER_SIZE;
    ret = teaclave_register_input_file_serialized(client0, user0_serialized_input_request, serialized_response,
                                                  &serialized_response_len);
    if (ret != 0) {
        fprintf(stderr, "[-] %s Failed to register input data.\n", user0_data.user_id);
        goto bail;
    }
    sscanf(serialized_response, "{\"data_id\":\"%42s", user0_input_id);

    /* User0 register output data. */
    printf("[+] %s register output data\n", user0_data.user_id);
    snprintf(user0_serialized_output_request, BUFFER_SIZE, register_output_serialized, user0_data.output_url);
    memset(serialized_response, 0, BUFFER_SIZE);
    serialized_response_len = BUFFER_SIZE;
    ret = teaclave_register_output_file_serialized(client0, user0_serialized_output_request, serialized_response,
                                                   &serialized_response_len);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to register output data.\n");
        goto bail;
    }
    sscanf(serialized_response, "{\"data_id\":\"%43s", user0_output_id);

    /* User0 assign data. */
    snprintf(user0_serialized_assign_request, BUFFER_SIZE, user0_assign_serialized, task_id, user0_input_id,
             user0_output_id);
    memset(serialized_response, 0, BUFFER_SIZE);
    serialized_response_len = BUFFER_SIZE;
    printf("[+] %s assign data\n", user0_data.user_id);
    ret = teaclave_assign_data_serialized(client0, user0_serialized_assign_request, serialized_response,
                                          &serialized_response_len);
    if (ret != 0) {
        fprintf(stderr, "[-] %s failed to assign data.\n", user0_data.user_id);
        goto bail;
    }

    /* User1 register input data. */
    printf("[+] %s register input data\n", user1_data.user_id);
    snprintf(user1_serialized_input_request, BUFFER_SIZE, register_input_serialized, user1_data.input_url,
         format_cmac_to_string(user1_data));
    memset(serialized_response, 0, BUFFER_SIZE);
    serialized_response_len = BUFFER_SIZE;
    ret = teaclave_register_input_file_serialized(client1, user1_serialized_input_request, serialized_response,
                                                  &serialized_response_len);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to register input data.\n");
        goto bail;
    }
    sscanf(serialized_response, "{\"data_id\":\"%42s", user1_input_id);

    /* User1 register output data. */
    printf("[+] %s register output data\n", user1_data.user_id);
    snprintf(user1_serialized_output_request, BUFFER_SIZE, register_output_serialized, user1_data.output_url);
    memset(serialized_response, 0, BUFFER_SIZE);
    serialized_response_len = BUFFER_SIZE;
    ret = teaclave_register_output_file_serialized(client1, user1_serialized_output_request, serialized_response,
                                                   &serialized_response_len);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to register output data. %d\n", ret);
        goto bail;
    }
    sscanf(serialized_response, "{\"data_id\":\"%43s", user1_output_id);

    /* User1 assign  data. */
    printf("[+] %s assign data\n", user1_data.user_id);
    snprintf(user1_serialized_assign_request, BUFFER_SIZE, user1_assign_serialized, task_id, user1_input_id,
             user1_output_id);
    memset(serialized_response, 0, BUFFER_SIZE);
    serialized_response_len = BUFFER_SIZE;
    ret = teaclave_assign_data_serialized(client1, user1_serialized_assign_request, serialized_response,
                                          &serialized_response_len);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to assign data.\n");
        goto bail;
    }

    /* User0 approve task. */
    printf("[+] user0 approve task \n");
    snprintf(user0_approve_request, BUFFER_SIZE, approve_serialized, task_id);
    memset(serialized_response, 0, BUFFER_SIZE);
    serialized_response_len = BUFFER_SIZE;
    ret =
        teaclave_approve_task_serialized(client0, user0_approve_request, serialized_response, &serialized_response_len);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to approve task.\n");
        goto bail;
    }

    /* User1 approve task. */
    printf("[+] user1 approve task \n");
    snprintf(user1_approve_request, BUFFER_SIZE, approve_serialized, task_id);
    memset(serialized_response, 0, BUFFER_SIZE);
    serialized_response_len = BUFFER_SIZE;
    ret =
        teaclave_approve_task_serialized(client1, user1_approve_request, serialized_response, &serialized_response_len);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to approve task.\n");
        goto bail;
    }

    /* User0 Invoke task. */
    printf("[+] user0 invoke task \n");
    ret = teaclave_invoke_task(client0, task_id);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to invoke the task.\n");
        goto bail;
    }

    /* User0 get task result. */
    printf("[+] user0 get task result \n");
    size_t user0_task_result_len = BUFFER_SIZE;
    ret = teaclave_get_task_result(client0, task_id, user0_task_result, &user0_task_result_len);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to get the task result.\n");
        goto bail;
    }
    printf("[+] %s task result in string: %s\n", user0_data.user_id, user0_task_result);

    /* User1 get task result. */
    printf("[+] user1 get task result \n");
    size_t user1_task_result_len = BUFFER_SIZE;
    ret = teaclave_get_task_result(client1, task_id, user1_task_result, &user1_task_result_len);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to get the task result.\n");
        goto bail;
    }
    printf("[+] %s task result in string: %s\n", user1_data.user_id, user1_task_result);

    printf("close client - 0\n");
    ret = teaclave_close_frontend_service(client0);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to close the frontend service client.\n");
    }
    printf("close client - 1\n");
    ret = teaclave_close_frontend_service(client1);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to close the frontend service client.\n");
    }
    return ret;

bail:
    printf("close client - 0\n");
    ret = teaclave_close_frontend_service(client0);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to close the frontend service client.\n");
    }
    printf("close client - 1\n");
    ret = teaclave_close_frontend_service(client1);
    if (ret != 0) {
        fprintf(stderr, "[-] Failed to close the frontend service client.\n");
    }
    exit(-1);
}
