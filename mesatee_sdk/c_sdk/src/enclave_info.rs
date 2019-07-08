// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use libc::c_char;
use mesatee_sdk::MesateeEnclaveInfo;
use std::{ffi, ptr};

impl OpaquePointerType for MesateeEnclaveInfo {}

#[no_mangle]
unsafe extern "C" fn mesatee_enclave_info_load(
    auditors_ptr: *const MesateeAuditorSet,
    enclave_info_file_path_ptr: *const c_char,
) -> *mut MesateeEnclaveInfo {
    check_inner_result!(
        inner_mesatee_enclave_info_load(auditors_ptr, enclave_info_file_path_ptr),
        ptr::null_mut()
    )
}

unsafe fn inner_mesatee_enclave_info_load(
    auditors_ptr: *const MesateeAuditorSet,
    enclave_info_file_path_ptr: *const c_char,
) -> MesateeResult<*mut MesateeEnclaveInfo> {
    let auditors = sanitize_const_ptr_for_ref(auditors_ptr)?;

    let mut enclave_auditors: Vec<(&str, &str)> = Vec::new();
    for (pub_key_path, sig_path) in auditors.inner.iter() {
        let pub_key_path_str = pub_key_path
            .to_str()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        let sig_path_str = sig_path
            .to_str()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        enclave_auditors.push((pub_key_path_str, sig_path_str));
    }

    if enclave_info_file_path_ptr.is_null() {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    let enclave_info_file_path = ffi::CStr::from_ptr(enclave_info_file_path_ptr)
        .to_str()
        .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;

    let enclave_info = MesateeEnclaveInfo::load(enclave_auditors, enclave_info_file_path)?;
    let enclave_info_ptr = Box::into_raw(Box::new(enclave_info)) as *mut MesateeEnclaveInfo;
    Ok(enclave_info_ptr)
}

#[no_mangle]
unsafe extern "C" fn mesatee_enclave_info_free(enclave_info_ptr: *mut MesateeEnclaveInfo) -> c_int {
    check_inner_result!(
        inner_mesatee_enclave_info_free(enclave_info_ptr),
        MESATEE_ERROR
    )
}

unsafe fn inner_mesatee_enclave_info_free(
    enclave_info_ptr: *mut MesateeEnclaveInfo,
) -> MesateeResult<c_int> {
    let _ = sanitize_ptr_for_mut_ref(enclave_info_ptr)?;
    let _ = Box::from_raw(enclave_info_ptr);
    Ok(MESATEE_SUCCESS)
}
