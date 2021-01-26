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

use libc::size_t;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::ptr;

use crate::{
    AuthenticationClient, AuthenticationService, EnclaveInfo, FrontendClient, FrontendService,
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

#[no_mangle]
pub extern "C" fn teaclave_connect_authentication_service(
    address: *const c_char,
    enclave_info_path: *const c_char,
    as_root_ca_cert_path: *const c_char,
) -> *mut AuthenticationClient {
    if address.is_null() || enclave_info_path.is_null() || as_root_ca_cert_path.is_null() {
        return ptr::null_mut();
    }

    let address = unsafe { CStr::from_ptr(address).to_string_lossy().into_owned() };
    let enclave_info_path = unsafe {
        CStr::from_ptr(enclave_info_path)
            .to_string_lossy()
            .into_owned()
    };
    let as_root_ca_cert_path = unsafe {
        CStr::from_ptr(as_root_ca_cert_path)
            .to_string_lossy()
            .into_owned()
    };
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

#[no_mangle]
pub unsafe extern "C" fn teaclave_close_authentication_service(
    client: *mut AuthenticationClient,
) -> c_int {
    if client.is_null() {
        return 1;
    }

    Box::from_raw(client);

    0
}

#[no_mangle]
pub extern "C" fn teaclave_user_register(
    client: &mut AuthenticationClient,
    user_id: *const c_char,
    user_password: *const c_char,
) -> c_int {
    if (client as *mut AuthenticationClient).is_null()
        || user_id.is_null()
        || user_password.is_null()
    {
        return 1;
    }

    let user_id = unsafe { CStr::from_ptr(user_id).to_string_lossy().into_owned() };
    let user_password = unsafe { CStr::from_ptr(user_password).to_string_lossy().into_owned() };
    unwrap_or_return_one!(client.user_register(&user_id, &user_password));

    0
}

#[no_mangle]
pub extern "C" fn teaclave_user_login(
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

    let user_id = unsafe { CStr::from_ptr(user_id).to_string_lossy().into_owned() };
    let user_password = unsafe { CStr::from_ptr(user_password).to_string_lossy().into_owned() };

    let token_string = unwrap_or_return_one!(client.user_login(&user_id, &user_password));
    let token_c_string = unwrap_or_return_one!(CString::new(token_string));
    let bytes = token_c_string.as_bytes_with_nul();

    unsafe {
        if *token_len < bytes.len() {
            return 1;
        } else {
            ptr::copy_nonoverlapping(bytes.as_ptr(), token as _, bytes.len());
            *token_len = bytes.len();
        }
    }

    0
}

#[no_mangle]
pub extern "C" fn teaclave_connect_frontend_service(
    address: *const c_char,
    enclave_info_path: *const c_char,
    as_root_ca_cert_path: *const c_char,
) -> *mut FrontendClient {
    if address.is_null() || enclave_info_path.is_null() || as_root_ca_cert_path.is_null() {
        return ptr::null_mut();
    }

    let address = unsafe { CStr::from_ptr(address).to_string_lossy().into_owned() };
    let enclave_info_path = unsafe {
        CStr::from_ptr(enclave_info_path)
            .to_string_lossy()
            .into_owned()
    };
    let as_root_ca_cert_path = unsafe {
        CStr::from_ptr(as_root_ca_cert_path)
            .to_string_lossy()
            .into_owned()
    };
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

#[no_mangle]
pub unsafe extern "C" fn teaclave_close_frontend_service(client: *mut FrontendClient) -> c_int {
    if client.is_null() {
        return 1;
    }

    Box::from_raw(client);

    0
}

#[no_mangle]
pub extern "C" fn teaclave_set_credential(
    client: &mut FrontendClient,
    user_id: *const c_char,
    user_token: *const c_char,
) -> c_int {
    if (client as *mut FrontendClient).is_null() || user_id.is_null() || user_token.is_null() {
        return 1;
    }

    let user_id = unsafe { CStr::from_ptr(user_id).to_string_lossy().into_owned() };
    let user_token = unsafe { CStr::from_ptr(user_token).to_string_lossy().into_owned() };
    client.set_credential(&user_id, &user_token);

    0
}

#[no_mangle]
pub extern "C" fn teaclave_invoke_task(
    client: &mut FrontendClient,
    task_id: *const c_char,
) -> c_int {
    if (client as *mut FrontendClient).is_null() || task_id.is_null() {
        return 1;
    }

    let task_id = unsafe { CStr::from_ptr(task_id).to_string_lossy().into_owned() };
    match client.invoke_task(&task_id) {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

#[no_mangle]
pub extern "C" fn teaclave_get_task_result(
    client: &mut FrontendClient,
    task_id: *const c_char,
    task_result: *mut c_char,
    task_result_len: *mut size_t,
) -> c_int {
    if (client as *mut FrontendClient).is_null() || task_id.is_null() {
        return 1;
    }

    let task_id = unsafe { CStr::from_ptr(task_id).to_string_lossy().into_owned() };
    match client.get_task_result(&task_id) {
        Ok(result) => {
            unsafe {
                if *task_result_len < result.len() {
                    return 1;
                } else {
                    ptr::copy_nonoverlapping(result.as_ptr(), task_result as _, result.len());
                    *task_result_len = result.len();
                }
            }
            0
        }
        Err(_) => 1,
    }
}

macro_rules! generate_function_serialized {
    ( $client_type:ident, $c_function_name:ident, $rust_function_name:ident) => {
        #[no_mangle]
        pub extern "C" fn $c_function_name(
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

            let serialized_request = unsafe {
                CStr::from_ptr(serialized_request)
                    .to_string_lossy()
                    .into_owned()
            };
            let function_id_string =
                unwrap_or_return_one!(client.$rust_function_name(&serialized_request));
            let function_id_c_string = unwrap_or_return_one!(CString::new(function_id_string));
            let bytes = function_id_c_string.as_bytes_with_nul();

            unsafe {
                if *serialized_response_len < bytes.len() {
                    return 1;
                } else {
                    ptr::copy_nonoverlapping(bytes.as_ptr(), serialized_response as _, bytes.len());
                    *serialized_response_len = bytes.len();
                }
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
    teaclave_get_task_serialized,
    get_task_serialized
);
