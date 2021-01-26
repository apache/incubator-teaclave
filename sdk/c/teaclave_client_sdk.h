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
 */


/* DO NOT MODIFY THIS MANUALLY! This file was generated using cbindgen.
 * To generate this file:
 * 1. Get the latest cbindgen using `cargo install --force cbindgen`
 * 2. Run `rustup run nightly cbindgen ../rust -c cbindgen.toml -o
     teaclave_client_sdk.h` or `make`.
 */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct AuthenticationClient AuthenticationClient;

typedef struct FrontendClient FrontendClient;

struct AuthenticationClient *teaclave_connect_authentication_service(const char *address,
                                                                     const char *enclave_info_path,
                                                                     const char *as_root_ca_cert_path);

int teaclave_close_authentication_service(struct AuthenticationClient *client);

int teaclave_user_register(struct AuthenticationClient *client,
                           const char *user_id,
                           const char *user_password);

int teaclave_user_login(struct AuthenticationClient *client,
                        const char *user_id,
                        const char *user_password,
                        char *token,
                        size_t *token_len);

struct FrontendClient *teaclave_connect_frontend_service(const char *address,
                                                         const char *enclave_info_path,
                                                         const char *as_root_ca_cert_path);

int teaclave_close_frontend_service(struct FrontendClient *client);

int teaclave_set_credential(struct FrontendClient *client,
                            const char *user_id,
                            const char *user_token);

int teaclave_invoke_task(struct FrontendClient *client, const char *task_id);

int teaclave_get_task_result(struct FrontendClient *client,
                             const char *task_id,
                             char *task_result,
                             size_t *task_result_len);

int teaclave_user_register_serialized(struct AuthenticationClient *client,
                                      const char *serialized_request,
                                      char *serialized_response,
                                      size_t *serialized_response_len);

int teaclave_user_login_serialized(struct AuthenticationClient *client,
                                   const char *serialized_request,
                                   char *serialized_response,
                                   size_t *serialized_response_len);

int teaclave_register_function_serialized(struct FrontendClient *client,
                                          const char *serialized_request,
                                          char *serialized_response,
                                          size_t *serialized_response_len);

int teaclave_get_function_serialized(struct FrontendClient *client,
                                     const char *serialized_request,
                                     char *serialized_response,
                                     size_t *serialized_response_len);

int teaclave_register_input_file_serialized(struct FrontendClient *client,
                                            const char *serialized_request,
                                            char *serialized_response,
                                            size_t *serialized_response_len);

int teaclave_register_output_file_serialized(struct FrontendClient *client,
                                             const char *serialized_request,
                                             char *serialized_response,
                                             size_t *serialized_response_len);

int teaclave_create_task_serialized(struct FrontendClient *client,
                                    const char *serialized_request,
                                    char *serialized_response,
                                    size_t *serialized_response_len);

int teaclave_assign_data_serialized(struct FrontendClient *client,
                                    const char *serialized_request,
                                    char *serialized_response,
                                    size_t *serialized_response_len);

int teaclave_approve_task_serialized(struct FrontendClient *client,
                                     const char *serialized_request,
                                     char *serialized_response,
                                     size_t *serialized_response_len);

int teaclave_invoke_task_serialized(struct FrontendClient *client,
                                    const char *serialized_request,
                                    char *serialized_response,
                                    size_t *serialized_response_len);

int teaclave_get_task_serialized(struct FrontendClient *client,
                                 const char *serialized_request,
                                 char *serialized_response,
                                 size_t *serialized_response_len);
