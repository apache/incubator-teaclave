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

/**
 * Connect to Teaclave Authentication Service.
 *
 * This function connects and establishes trusted channel to the remote
 * authentication service with `address`. The function gets *enclave info* from
 * the file located in `enclave_info_path`. The file in `as_root_ca_cert_path`
 * will be used to verify the attestation report from the remote service.
 *
 * # Arguments
 *
 * * `address`: address of remote services, normally,it's a domain name with port number.
 * * `enclave_info_path`: file path of the *enclave info* TOML file.
 * * `as_root_ca_cert_path`: file path of the certificate for remote attestation service.
 *
 * # Return
 *
 * * The function returns an opaque pointer (handle) of the service. On error,
 * the function returns NULL.
 *
 * # Safety
 *
 * `address`, `enclave_info_path`, `as_root_ca_cert_path` should be C string (null terminated).
 */
struct AuthenticationClient *teaclave_connect_authentication_service(const char *address,
                                                                     const char *enclave_info_path,
                                                                     const char *as_root_ca_cert_path);

/**
 * Close and free the authentication service handle, i.e., the
 * `AuthenticaionClient` type opaque pointer. The function returns 0 for
 * success. On error, the function returns 1.
 *
 * # Safety
 *
 * This function is unsafe because improper use may lead to
 * memory problems. For example, a double-free may occur if the
 * function is called twice on the same raw pointer.
 */
int teaclave_close_authentication_service(struct AuthenticationClient *client);

/**
 * Register a new user with `user_id` and `user_password`. The function returns
 * 0 for success. On error, the function returns 1.
 *
 * # Safety
 *
 * `user_id`, `user_password`, `role`, `attribute` should be C string (null terminated).
 */
int teaclave_user_register(struct AuthenticationClient *client,
                           const char *user_id,
                           const char *user_password,
                           const char *role,
                           const char *attribute);

/**
 * Login a new user with `user_id` and `user_password`. The login session token
 * will be save in the `token` buffer, and length will be set in the
 * `token_len` argument. The function returns 0 for success. On error, the
 * function returns 1.
 *
 * # Safety
 *
 * `user_id`, `user_password` should be C string (null terminated), token and token_len should be consistent.
 */
int teaclave_user_login(struct AuthenticationClient *client,
                        const char *user_id,
                        const char *user_password,
                        char *token,
                        size_t *token_len);

/**
 * Connect to Teaclave Frontend Service.
 *
 * This function connects and establishes trusted channel to the remote
 * frontend service with `address`. The function gets *enclave info* from
 * the file located in `enclave_info_path`. The file in `as_root_ca_cert_path`
 * will be used to verify the attestation report from the remote service.
 *
 * # Arguments
 *
 * * `address`: address of remote services, normally,it's a domain name with port number.
 * * `enclave_info_path`: file path of the *enclave info* TOML file.
 * * `as_root_ca_cert_path`: file path of the certificate for remote attestation service.
 *
 * # Return
 *
 * * The function returns an opaque pointer (handle) of the service. On error,
 * the function returns NULL.
 *
 * # Safety
 *
 * All arguments should be C string (null terminated).
 */
struct FrontendClient *teaclave_connect_frontend_service(const char *address,
                                                         const char *enclave_info_path,
                                                         const char *as_root_ca_cert_path);

/**
 * Close and free the frontend service handle, i.e., the `FrontendClient` type
 * opaque pointer. The function returns 0 for success. On error, the function
 * returns 1.
 *
 * # Safety
 *
 * This function is unsafe because improper use may lead to
 * memory problems. For example, a double-free may occur if the
 * function is called twice on the same raw pointer.
 */
int teaclave_close_frontend_service(struct FrontendClient *client);

/**
 * Set user's credential with `user_id` and `user_token`. The function returns
 * 0 for success. On error, the function returns 1.
 *
 * # Safety
 *
 * `user_id` and `user_token` should be C string (null terminated).
 */
int teaclave_authentication_set_credential(struct AuthenticationClient *client,
                                           const char *user_id,
                                           const char *user_token);

/**
 * Set user's credential with `user_id` and `user_token`. The function returns
 * 0 for success. On error, the function returns 1.
 *
 * # Safety
 *
 * `user_id` and `user_token` should be C string (null terminated).
 */
int teaclave_frontend_set_credential(struct FrontendClient *client,
                                     const char *user_id,
                                     const char *user_token);

/**
 * Invoke task with `task_id`. The function returns 0 for success. On error,
 * the function returns 1.
 *
 * # Safety
 *
 * `task_id` should be C string (null terminated).
 */
int teaclave_invoke_task(struct FrontendClient *client, const char *task_id);

/**
 * Cancel task with `task_id`. The function returns 0 for success. On error,
 * the function returns 1.
 *
 * # Safety
 *
 * `task_id` should be C string (null terminated).
 */
int teaclave_cancel_task(struct FrontendClient *client, const char *task_id);

/**
 * Get task result of `task_id`. The result will be save in the `task_result`
 * buffer, and set corresponding `task_result_len` argument. Note that this is
 * a blocking function and wait for the return of the task. The function
 * returns 0 for success. On error, the function returns 1.
 *
 * # Safety
 *
 * Inconsistent length of allocated buffer may caused overflow.
 */
int teaclave_get_task_result(struct FrontendClient *client,
                             const char *task_id,
                             char *task_result,
                             size_t *task_result_len);

/**
 * Send JSON serialized request to the service with the `client` and
 * get the serialized response.
 *
 * # Arguments
 *
 * * `client`: service client.
 * * `serialized_request`; JSON serialized request
 * * `serialized_response`: buffer to store the JSON serialized response.
 * * `serialized_response_len`: length of the allocated
 *   `serialized_response`, will be set as the length of
 *   `serialized_response` when return successfully.
 *
 * # Return
 *
 * The function returns 0 for success. On error, the function returns 1.
 *
 * # Safety
 *
 * Inconsistent length of allocated buffer may caused overflow.
 */
int teaclave_user_register_serialized(struct AuthenticationClient *client,
                                      const char *serialized_request,
                                      char *serialized_response,
                                      size_t *serialized_response_len);

/**
 * Send JSON serialized request to the service with the `client` and
 * get the serialized response.
 *
 * # Arguments
 *
 * * `client`: service client.
 * * `serialized_request`; JSON serialized request
 * * `serialized_response`: buffer to store the JSON serialized response.
 * * `serialized_response_len`: length of the allocated
 *   `serialized_response`, will be set as the length of
 *   `serialized_response` when return successfully.
 *
 * # Return
 *
 * The function returns 0 for success. On error, the function returns 1.
 *
 * # Safety
 *
 * Inconsistent length of allocated buffer may caused overflow.
 */
int teaclave_user_login_serialized(struct AuthenticationClient *client,
                                   const char *serialized_request,
                                   char *serialized_response,
                                   size_t *serialized_response_len);

/**
 * Send JSON serialized request to the service with the `client` and
 * get the serialized response.
 *
 * # Arguments
 *
 * * `client`: service client.
 * * `serialized_request`; JSON serialized request
 * * `serialized_response`: buffer to store the JSON serialized response.
 * * `serialized_response_len`: length of the allocated
 *   `serialized_response`, will be set as the length of
 *   `serialized_response` when return successfully.
 *
 * # Return
 *
 * The function returns 0 for success. On error, the function returns 1.
 *
 * # Safety
 *
 * Inconsistent length of allocated buffer may caused overflow.
 */
int teaclave_register_function_serialized(struct FrontendClient *client,
                                          const char *serialized_request,
                                          char *serialized_response,
                                          size_t *serialized_response_len);

/**
 * Send JSON serialized request to the service with the `client` and
 * get the serialized response.
 *
 * # Arguments
 *
 * * `client`: service client.
 * * `serialized_request`; JSON serialized request
 * * `serialized_response`: buffer to store the JSON serialized response.
 * * `serialized_response_len`: length of the allocated
 *   `serialized_response`, will be set as the length of
 *   `serialized_response` when return successfully.
 *
 * # Return
 *
 * The function returns 0 for success. On error, the function returns 1.
 *
 * # Safety
 *
 * Inconsistent length of allocated buffer may caused overflow.
 */
int teaclave_get_function_serialized(struct FrontendClient *client,
                                     const char *serialized_request,
                                     char *serialized_response,
                                     size_t *serialized_response_len);

/**
 * Send JSON serialized request to the service with the `client` and
 * get the serialized response.
 *
 * # Arguments
 *
 * * `client`: service client.
 * * `serialized_request`; JSON serialized request
 * * `serialized_response`: buffer to store the JSON serialized response.
 * * `serialized_response_len`: length of the allocated
 *   `serialized_response`, will be set as the length of
 *   `serialized_response` when return successfully.
 *
 * # Return
 *
 * The function returns 0 for success. On error, the function returns 1.
 *
 * # Safety
 *
 * Inconsistent length of allocated buffer may caused overflow.
 */
int teaclave_register_input_file_serialized(struct FrontendClient *client,
                                            const char *serialized_request,
                                            char *serialized_response,
                                            size_t *serialized_response_len);

/**
 * Send JSON serialized request to the service with the `client` and
 * get the serialized response.
 *
 * # Arguments
 *
 * * `client`: service client.
 * * `serialized_request`; JSON serialized request
 * * `serialized_response`: buffer to store the JSON serialized response.
 * * `serialized_response_len`: length of the allocated
 *   `serialized_response`, will be set as the length of
 *   `serialized_response` when return successfully.
 *
 * # Return
 *
 * The function returns 0 for success. On error, the function returns 1.
 *
 * # Safety
 *
 * Inconsistent length of allocated buffer may caused overflow.
 */
int teaclave_register_output_file_serialized(struct FrontendClient *client,
                                             const char *serialized_request,
                                             char *serialized_response,
                                             size_t *serialized_response_len);

/**
 * Send JSON serialized request to the service with the `client` and
 * get the serialized response.
 *
 * # Arguments
 *
 * * `client`: service client.
 * * `serialized_request`; JSON serialized request
 * * `serialized_response`: buffer to store the JSON serialized response.
 * * `serialized_response_len`: length of the allocated
 *   `serialized_response`, will be set as the length of
 *   `serialized_response` when return successfully.
 *
 * # Return
 *
 * The function returns 0 for success. On error, the function returns 1.
 *
 * # Safety
 *
 * Inconsistent length of allocated buffer may caused overflow.
 */
int teaclave_create_task_serialized(struct FrontendClient *client,
                                    const char *serialized_request,
                                    char *serialized_response,
                                    size_t *serialized_response_len);

/**
 * Send JSON serialized request to the service with the `client` and
 * get the serialized response.
 *
 * # Arguments
 *
 * * `client`: service client.
 * * `serialized_request`; JSON serialized request
 * * `serialized_response`: buffer to store the JSON serialized response.
 * * `serialized_response_len`: length of the allocated
 *   `serialized_response`, will be set as the length of
 *   `serialized_response` when return successfully.
 *
 * # Return
 *
 * The function returns 0 for success. On error, the function returns 1.
 *
 * # Safety
 *
 * Inconsistent length of allocated buffer may caused overflow.
 */
int teaclave_assign_data_serialized(struct FrontendClient *client,
                                    const char *serialized_request,
                                    char *serialized_response,
                                    size_t *serialized_response_len);

/**
 * Send JSON serialized request to the service with the `client` and
 * get the serialized response.
 *
 * # Arguments
 *
 * * `client`: service client.
 * * `serialized_request`; JSON serialized request
 * * `serialized_response`: buffer to store the JSON serialized response.
 * * `serialized_response_len`: length of the allocated
 *   `serialized_response`, will be set as the length of
 *   `serialized_response` when return successfully.
 *
 * # Return
 *
 * The function returns 0 for success. On error, the function returns 1.
 *
 * # Safety
 *
 * Inconsistent length of allocated buffer may caused overflow.
 */
int teaclave_approve_task_serialized(struct FrontendClient *client,
                                     const char *serialized_request,
                                     char *serialized_response,
                                     size_t *serialized_response_len);

/**
 * Send JSON serialized request to the service with the `client` and
 * get the serialized response.
 *
 * # Arguments
 *
 * * `client`: service client.
 * * `serialized_request`; JSON serialized request
 * * `serialized_response`: buffer to store the JSON serialized response.
 * * `serialized_response_len`: length of the allocated
 *   `serialized_response`, will be set as the length of
 *   `serialized_response` when return successfully.
 *
 * # Return
 *
 * The function returns 0 for success. On error, the function returns 1.
 *
 * # Safety
 *
 * Inconsistent length of allocated buffer may caused overflow.
 */
int teaclave_invoke_task_serialized(struct FrontendClient *client,
                                    const char *serialized_request,
                                    char *serialized_response,
                                    size_t *serialized_response_len);

/**
 * Send JSON serialized request to the service with the `client` and
 * get the serialized response.
 *
 * # Arguments
 *
 * * `client`: service client.
 * * `serialized_request`; JSON serialized request
 * * `serialized_response`: buffer to store the JSON serialized response.
 * * `serialized_response_len`: length of the allocated
 *   `serialized_response`, will be set as the length of
 *   `serialized_response` when return successfully.
 *
 * # Return
 *
 * The function returns 0 for success. On error, the function returns 1.
 *
 * # Safety
 *
 * Inconsistent length of allocated buffer may caused overflow.
 */
int teaclave_cancel_task_serialized(struct FrontendClient *client,
                                    const char *serialized_request,
                                    char *serialized_response,
                                    size_t *serialized_response_len);

/**
 * Send JSON serialized request to the service with the `client` and
 * get the serialized response.
 *
 * # Arguments
 *
 * * `client`: service client.
 * * `serialized_request`; JSON serialized request
 * * `serialized_response`: buffer to store the JSON serialized response.
 * * `serialized_response_len`: length of the allocated
 *   `serialized_response`, will be set as the length of
 *   `serialized_response` when return successfully.
 *
 * # Return
 *
 * The function returns 0 for success. On error, the function returns 1.
 *
 * # Safety
 *
 * Inconsistent length of allocated buffer may caused overflow.
 */
int teaclave_get_task_serialized(struct FrontendClient *client,
                                 const char *serialized_request,
                                 char *serialized_response,
                                 size_t *serialized_response_len);
