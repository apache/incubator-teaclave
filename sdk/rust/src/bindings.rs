// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use libc::{c_char, c_int, c_void, malloc, size_t};

use std::ffi::{CStr, CString};
use std::{fs, ptr};

use crate::{
    AuthenticationClient, AuthenticationService, EnclaveInfo, Entry, FrontendClient,
    FrontendService,
};

macro_rules! unwrap_or_return_null {
    ( $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => return ptr::null_mut(),
        }
    };
}

macro_rules! unwrap_or_return_one {
    ( $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => return 1,
        }
    };
}

/// Connect to Teaclave Authentication Service.
///
/// This function connects and establishes trusted channel to the remote
/// authentication service with `address`. The function gets *enclave info* from
/// the file located in `enclave_info_path`. The file in `as_root_ca_cert_path`
/// will be used to verify the attestation report from the remote service.
///
/// # Arguments
///
/// * `address`: address of remote services, normally,it's a domain name with port number.
/// * `enclave_info_path`: file path of the *enclave info* TOML file.
/// * `as_root_ca_cert_path`: file path of the certificate for remote attestation service.
///
/// # Return
///
/// * The function returns an opaque pointer (handle) of the service. On error,
/// the function returns NULL.
///
/// # Safety
///
/// `address`, `enclave_info_path`, `as_root_ca_cert_path` should be C string (null terminated).
#[no_mangle]
pub unsafe extern "C" fn teaclave_connect_authentication_service(
    address: *const c_char,
    enclave_info_path: *const c_char,
    as_root_ca_cert_path: *const c_char,
) -> *mut AuthenticationClient {
    if address.is_null() || enclave_info_path.is_null() || as_root_ca_cert_path.is_null() {
        return ptr::null_mut();
    }

    let address = CStr::from_ptr(address).to_string_lossy().into_owned();
    let enclave_info_path = CStr::from_ptr(enclave_info_path)
        .to_string_lossy()
        .into_owned();
    let as_root_ca_cert_path = CStr::from_ptr(as_root_ca_cert_path)
        .to_string_lossy()
        .into_owned();
    let enclave_info = unwrap_or_return_null!(EnclaveInfo::from_file(enclave_info_path));
    let bytes = unwrap_or_return_null!(fs::read(as_root_ca_cert_path));
    let as_root_ca_cert = unwrap_or_return_null!(pem::parse(bytes)).contents;
    let client = unwrap_or_return_null!(AuthenticationService::connect(
        &address,
        &enclave_info,
        &as_root_ca_cert
    ));

    Box::into_raw(Box::new(client))
}

/// Close and free the authentication service handle, i.e., the
/// `AuthenticaionClient` type opaque pointer. The function returns 0 for
/// success. On error, the function returns 1.
///
/// # Safety
///
/// This function is unsafe because improper use may lead to
/// memory problems. For example, a double-free may occur if the
/// function is called twice on the same raw pointer.
#[no_mangle]
pub unsafe extern "C" fn teaclave_close_authentication_service(
    client: *mut AuthenticationClient,
) -> c_int {
    if client.is_null() {
        return 1;
    }

    let _ = Box::from_raw(client);

    0
}

/// Register a new user with `user_id` and `user_password`. The function returns
/// 0 for success. On error, the function returns 1.
///
/// # Safety
///
/// `user_id`, `user_password`, `role`, `attribute` should be C string (null terminated).
#[no_mangle]
pub unsafe extern "C" fn teaclave_user_register(
    client: &mut AuthenticationClient,
    user_id: *const c_char,
    user_password: *const c_char,
    role: *const c_char,
    attribute: *const c_char,
) -> c_int {
    if (client as *mut AuthenticationClient).is_null()
        || user_id.is_null()
        || user_password.is_null()
    {
        return 1;
    }

    let user_id = CStr::from_ptr(user_id).to_string_lossy().into_owned();
    let user_password = CStr::from_ptr(user_password).to_string_lossy().into_owned();
    let role = CStr::from_ptr(role).to_string_lossy().into_owned();
    let attribute = CStr::from_ptr(attribute).to_string_lossy().into_owned();
    unwrap_or_return_one!(client.user_register(&user_id, &user_password, &role, &attribute));

    0
}

/// Login a new user with `user_id` and `user_password`. The login session token
/// will be save in the `token` buffer, and length will be set in the
/// `token_len` argument. The function returns 0 for success. On error, the
/// function returns 1.
///
/// # Safety
///
/// `user_id`, `user_password` should be C string (null terminated), token and token_len should be consistent.
#[no_mangle]
pub unsafe extern "C" fn teaclave_user_login(
    client: &mut AuthenticationClient,
    user_id: *const c_char,
    user_password: *const c_char,
    token: *mut c_char,
    token_len: *mut size_t,
) -> c_int {
    if (client as *mut AuthenticationClient).is_null()
        || user_id.is_null()
        || user_password.is_null()
        || token.is_null()
        || token_len.is_null()
    {
        return 1;
    }

    let user_id = CStr::from_ptr(user_id).to_string_lossy().into_owned();
    let user_password = CStr::from_ptr(user_password).to_string_lossy().into_owned();

    let token_string = unwrap_or_return_one!(client.user_login(&user_id, &user_password));
    let token_c_string = unwrap_or_return_one!(CString::new(token_string));
    let bytes = token_c_string.as_bytes_with_nul();

    if *token_len < bytes.len() {
        return 1;
    } else {
        ptr::copy_nonoverlapping(bytes.as_ptr(), token as _, bytes.len());
        *token_len = bytes.len();
    }

    0
}

/// Connect to Teaclave Frontend Service.
///
/// This function connects and establishes trusted channel to the remote
/// frontend service with `address`. The function gets *enclave info* from
/// the file located in `enclave_info_path`. The file in `as_root_ca_cert_path`
/// will be used to verify the attestation report from the remote service.
///
/// # Arguments
///
/// * `address`: address of remote services, normally,it's a domain name with port number.
/// * `enclave_info_path`: file path of the *enclave info* TOML file.
/// * `as_root_ca_cert_path`: file path of the certificate for remote attestation service.
///
/// # Return
///
/// * The function returns an opaque pointer (handle) of the service. On error,
/// the function returns NULL.
///
/// # Safety
///
/// All arguments should be C string (null terminated).
#[no_mangle]
pub unsafe extern "C" fn teaclave_connect_frontend_service(
    address: *const c_char,
    enclave_info_path: *const c_char,
    as_root_ca_cert_path: *const c_char,
) -> *mut FrontendClient {
    if address.is_null() || enclave_info_path.is_null() || as_root_ca_cert_path.is_null() {
        return ptr::null_mut();
    }

    let address = CStr::from_ptr(address).to_string_lossy().into_owned();
    let enclave_info_path = CStr::from_ptr(enclave_info_path)
        .to_string_lossy()
        .into_owned();
    let as_root_ca_cert_path = CStr::from_ptr(as_root_ca_cert_path)
        .to_string_lossy()
        .into_owned();
    let enclave_info = unwrap_or_return_null!(EnclaveInfo::from_file(enclave_info_path));
    let bytes = unwrap_or_return_null!(fs::read(as_root_ca_cert_path));
    let as_root_ca_cert = unwrap_or_return_null!(pem::parse(bytes)).contents;
    let client = unwrap_or_return_null!(FrontendService::connect(
        &address,
        &enclave_info,
        &as_root_ca_cert
    ));

    Box::into_raw(Box::new(client))
}

/// Close and free the frontend service handle, i.e., the `FrontendClient` type
/// opaque pointer. The function returns 0 for success. On error, the function
/// returns 1.
///
/// # Safety
///
/// This function is unsafe because improper use may lead to
/// memory problems. For example, a double-free may occur if the
/// function is called twice on the same raw pointer.
#[no_mangle]
pub unsafe extern "C" fn teaclave_close_frontend_service(client: *mut FrontendClient) -> c_int {
    if client.is_null() {
        return 1;
    }

    let _ = Box::from_raw(client);

    0
}

/// Set user's credential with `user_id` and `user_token`. The function returns
/// 0 for success. On error, the function returns 1.
///
/// # Safety
///
/// `user_id` and `user_token` should be C string (null terminated).
#[no_mangle]
pub unsafe extern "C" fn teaclave_authentication_set_credential(
    client: &mut AuthenticationClient,
    user_id: *const c_char,
    user_token: *const c_char,
) -> c_int {
    if (client as *mut AuthenticationClient).is_null() || user_id.is_null() || user_token.is_null()
    {
        return 1;
    }

    let user_id = CStr::from_ptr(user_id).to_string_lossy().into_owned();
    let user_token = CStr::from_ptr(user_token).to_string_lossy().into_owned();
    client.set_credential(&user_id, &user_token);

    0
}

/// Set user's credential with `user_id` and `user_token`. The function returns
/// 0 for success. On error, the function returns 1.
///
/// # Safety
///
/// `user_id` and `user_token` should be C string (null terminated).
#[no_mangle]
pub unsafe extern "C" fn teaclave_frontend_set_credential(
    client: &mut FrontendClient,
    user_id: *const c_char,
    user_token: *const c_char,
) -> c_int {
    if (client as *mut FrontendClient).is_null() || user_id.is_null() || user_token.is_null() {
        return 1;
    }

    let user_id = CStr::from_ptr(user_id).to_string_lossy().into_owned();
    let user_token = CStr::from_ptr(user_token).to_string_lossy().into_owned();
    client.set_credential(&user_id, &user_token);

    0
}

/// Invoke task with `task_id`. The function returns 0 for success. On error,
/// the function returns 1.
///
/// # Safety
///
/// `task_id` should be C string (null terminated).
#[no_mangle]
pub unsafe extern "C" fn teaclave_invoke_task(
    client: &mut FrontendClient,
    task_id: *const c_char,
) -> c_int {
    if (client as *mut FrontendClient).is_null() || task_id.is_null() {
        return 1;
    }

    let task_id = CStr::from_ptr(task_id).to_string_lossy().into_owned();
    match client.invoke_task(&task_id) {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

/// Cancel task with `task_id`. The function returns 0 for success. On error,
/// the function returns 1.
///
/// # Safety
///
/// `task_id` should be C string (null terminated).
#[no_mangle]
pub unsafe extern "C" fn teaclave_cancel_task(
    client: &mut FrontendClient,
    task_id: *const c_char,
) -> c_int {
    if (client as *mut FrontendClient).is_null() || task_id.is_null() {
        return 1;
    }

    let task_id = CStr::from_ptr(task_id).to_string_lossy().into_owned();
    match client.cancel_task(&task_id) {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

/// Get task result of `task_id`. The result will be save in the `task_result`
/// buffer, and set corresponding `task_result_len` argument. Note that this is
/// a blocking function and wait for the return of the task. The function
/// returns 0 for success. On error, the function returns 1.
///
/// # Safety
///
/// Inconsistent length of allocated buffer may caused overflow.
#[no_mangle]
pub unsafe extern "C" fn teaclave_get_task_result(
    client: &mut FrontendClient,
    task_id: *const c_char,
    task_result: *mut c_char,
    task_result_len: *mut size_t,
) -> c_int {
    if (client as *mut FrontendClient).is_null() || task_id.is_null() {
        return 1;
    }

    let task_id = CStr::from_ptr(task_id).to_string_lossy().into_owned();
    match client.get_task_result(&task_id) {
        Ok((result, _)) => {
            if *task_result_len < result.len() {
                return 1;
            } else {
                ptr::copy_nonoverlapping(result.as_ptr(), task_result as _, result.len());
                *task_result_len = result.len();
            }
            0
        }
        Err(_) => 1,
    }
}

/// Query audit logs according to `query`. `query` is the query statement for tantivy. The result
/// will be saved in the `log_buffer` buffer with the corresponding `log_len` argument set.
/// Remember to free the user and message inside c_entry to avoid memory leak.
///
/// The function returns 0 for success. On error, the function returns 1.
///
/// # Safety
///
/// Inconsistent length of allocated buffer may caused overflow.
#[no_mangle]
pub unsafe extern "C" fn teaclave_query_audit_logs(
    client: &mut FrontendClient,
    query: *const c_char,
    log_buffer: *mut c_entry,
    log_len: *mut size_t,
) -> c_int {
    if (client as *mut FrontendClient).is_null()
        || query.is_null()
        || log_buffer.is_null()
        || log_len.is_null()
        || *log_len == 0
    {
        return 1;
    }

    let query = CStr::from_ptr(query).to_string_lossy().into_owned();
    match client.query_audit_logs(query, *log_len) {
        Ok(audit_logs) => {
            let c_logs: Vec<_> = audit_logs.into_iter().map(c_entry::from).collect();
            let src_logs = c_logs.as_ptr();
            let len = c_logs.len();
            if len > *log_len {
                return 1;
            }

            unsafe {
                ptr::copy_nonoverlapping(src_logs, log_buffer, len);
            }

            *log_len = len;
            0
        }
        Err(_) => 1,
    }
}

#[repr(C)]
pub struct c_entry {
    microsecond: i64,
    ip: [u8; 16],
    user: *mut c_void,
    message: *mut c_void,
    result: bool,
}

impl From<Entry> for c_entry {
    fn from(entry: Entry) -> Self {
        let user_bytes = CString::new(entry.user()).unwrap().into_bytes_with_nul();
        let len = user_bytes.len();
        let user = unsafe {
            let user = malloc(len);
            ptr::copy_nonoverlapping(user_bytes.as_ptr(), user as *mut u8, len);
            user
        };

        let message_bytes = CString::new(entry.message()).unwrap().into_bytes_with_nul();
        let len = message_bytes.len();
        let message = unsafe {
            let message = malloc(len);
            ptr::copy_nonoverlapping(message_bytes.as_ptr(), message as *mut u8, len);
            message
        };

        Self {
            microsecond: entry.datetime().timestamp_micros(),
            ip: entry.ip().octets(),
            user,
            message,
            result: entry.result(),
        }
    }
}

macro_rules! generate_function_serialized {
    ( $client_type:ident, $c_function_name:ident, $rust_function_name:ident) => {
        /// Send JSON serialized request to the service with the `client` and
        /// get the serialized response.
        ///
        /// # Arguments
        ///
        /// * `client`: service client.
        /// * `serialized_request`; JSON serialized request
        /// * `serialized_response`: buffer to store the JSON serialized response.
        /// * `serialized_response_len`: length of the allocated
        ///   `serialized_response`, will be set as the length of
        ///   `serialized_response` when return successfully.
        ///
        /// # Return
        ///
        /// The function returns 0 for success. On error, the function returns 1.
        ///
        /// # Safety
        ///
        /// Inconsistent length of allocated buffer may caused overflow.
        #[no_mangle]
        pub unsafe extern "C" fn $c_function_name(
            client: &mut $client_type,
            serialized_request: *const c_char,
            serialized_response: *mut c_char,
            serialized_response_len: *mut size_t,
        ) -> c_int {
            if (client as *mut $client_type).is_null()
                || serialized_request.is_null()
                || serialized_response.is_null()
                || serialized_response_len.is_null()
            {
                return 1;
            }

            let serialized_request = CStr::from_ptr(serialized_request)
                .to_string_lossy()
                .into_owned();
            let function_id_string =
                unwrap_or_return_one!(client.$rust_function_name(&serialized_request));
            let function_id_c_string = unwrap_or_return_one!(CString::new(function_id_string));
            let bytes = function_id_c_string.as_bytes_with_nul();

            if *serialized_response_len < bytes.len() {
                return 1;
            } else {
                ptr::copy_nonoverlapping(bytes.as_ptr(), serialized_response as _, bytes.len());
                *serialized_response_len = bytes.len();
            }

            0
        }
    };
}

generate_function_serialized!(
    AuthenticationClient,
    teaclave_user_register_serialized,
    user_register_serialized
);
generate_function_serialized!(
    AuthenticationClient,
    teaclave_user_login_serialized,
    user_login_serialized
);
generate_function_serialized!(
    FrontendClient,
    teaclave_register_function_serialized,
    register_function_serialized
);
generate_function_serialized!(
    FrontendClient,
    teaclave_get_function_serialized,
    get_function_serialized
);
generate_function_serialized!(
    FrontendClient,
    teaclave_get_function_usage_stats_serialized,
    get_function_usage_stats_serialized
);
generate_function_serialized!(
    FrontendClient,
    teaclave_register_input_file_serialized,
    register_input_file_serialized
);
generate_function_serialized!(
    FrontendClient,
    teaclave_register_output_file_serialized,
    register_output_file_serialized
);
generate_function_serialized!(
    FrontendClient,
    teaclave_create_task_serialized,
    create_task_serialized
);
generate_function_serialized!(
    FrontendClient,
    teaclave_assign_data_serialized,
    assign_data_serialized
);
generate_function_serialized!(
    FrontendClient,
    teaclave_approve_task_serialized,
    approve_task_serialized
);
generate_function_serialized!(
    FrontendClient,
    teaclave_invoke_task_serialized,
    invoke_task_serialized
);
generate_function_serialized!(
    FrontendClient,
    teaclave_cancel_task_serialized,
    cancel_task_serialized
);
generate_function_serialized!(
    FrontendClient,
    teaclave_get_task_serialized,
    get_task_serialized
);
generate_function_serialized!(
    FrontendClient,
    teaclave_query_audit_logs_serialized,
    query_audit_logs_serialized
);
