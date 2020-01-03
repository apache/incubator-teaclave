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

use super::*;
use libc::c_char;
use std::ffi::CStr;
use std::path::{Path, PathBuf};

pub(super) struct MesateeAuditorSet {
    pub inner: Vec<(PathBuf, PathBuf)>,
}

impl OpaquePointerType for MesateeAuditorSet {}

impl MesateeAuditorSet {
    pub fn new() -> MesateeAuditorSet {
        MesateeAuditorSet { inner: Vec::new() }
    }
}

#[no_mangle]
unsafe extern "C" fn mesatee_auditor_set_new() -> *mut MesateeAuditorSet {
    Box::into_raw(Box::new(MesateeAuditorSet::new())) as *mut MesateeAuditorSet
}

#[no_mangle]
unsafe extern "C" fn mesatee_auditor_set_free(ptr: *mut MesateeAuditorSet) -> c_int {
    check_inner_result!(inner_mesatee_auditor_set_free(ptr), MESATEE_ERROR)
}

unsafe fn inner_mesatee_auditor_set_free(ptr: *mut MesateeAuditorSet) -> MesateeResult<c_int> {
    let _ = sanitize_ptr_for_mut_ref(ptr)?;
    let _ = Box::from_raw(ptr);
    Ok(MESATEE_SUCCESS)
}

#[no_mangle]
unsafe extern "C" fn mesatee_auditor_set_add_auditor(
    ptr: *mut MesateeAuditorSet,
    pub_key_path: *const c_char,
    sig_path: *const c_char,
) -> c_int {
    check_inner_result!(
        inner_mesatee_auditor_set_add_auditor(ptr, pub_key_path, sig_path),
        MESATEE_ERROR
    )
}

unsafe fn inner_mesatee_auditor_set_add_auditor(
    auditor_set_ptr: *mut MesateeAuditorSet,
    pub_key_path_ptr: *const c_char,
    sig_path_ptr: *const c_char,
) -> MesateeResult<c_int> {
    let auditor_set = sanitize_ptr_for_mut_ref(auditor_set_ptr)?;

    if pub_key_path_ptr.is_null() || pub_key_path_ptr.is_null() {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    let pub_key_path = CStr::from_ptr(pub_key_path_ptr)
        .to_str()
        .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
    let sig_path = CStr::from_ptr(sig_path_ptr)
        .to_str()
        .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;

    auditor_set.inner.push((
        Path::new(pub_key_path).to_path_buf(),
        Path::new(sig_path).to_path_buf(),
    ));
    Ok(MESATEE_SUCCESS)
}
