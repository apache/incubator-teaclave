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
use env_logger;
use libc::{c_char, c_int, sockaddr};
use mesatee_sdk::{Mesatee, MesateeEnclaveInfo};
use nix::sys::socket::SockAddr;
use std::{ffi, net, ptr};

impl OpaquePointerType for Mesatee {}

#[no_mangle]
pub unsafe extern "C" fn mesatee_init() -> c_int {
    env_logger::init();
    MESATEE_SUCCESS
}

#[no_mangle]
unsafe extern "C" fn mesatee_context_new(
    enclave_info_ptr: *mut MesateeEnclaveInfo,
    user_id: *const c_char,
    user_token: *const c_char,
    tms_addr: *mut sockaddr,
    tdfs_addr: *mut sockaddr,
) -> *mut Mesatee {
    check_inner_result!(
        inner_mesatee_context_new(enclave_info_ptr, user_id, user_token, tms_addr, tdfs_addr),
        ptr::null_mut()
    )
}

#[no_mangle]
unsafe extern "C" fn mesatee_context_new2(
    enclave_info_ptr: *mut MesateeEnclaveInfo,
    user_id: *const c_char,
    user_token: *const c_char,
    tms_addr_ptr: *const c_char,
    tdfs_addr_ptr: *const c_char,
) -> *mut Mesatee {
    check_inner_result!(
        inner_mesatee_context_new2(
            enclave_info_ptr,
            user_id,
            user_token,
            tms_addr_ptr,
            tdfs_addr_ptr
        ),
        ptr::null_mut()
    )
}

unsafe fn inner_mesatee_context_new(
    enclave_info_ptr: *mut MesateeEnclaveInfo,
    user_id_ptr: *const c_char,
    user_token_ptr: *const c_char,
    tms_addr_ptr: *const sockaddr,
    tdfs_addr_ptr: *const sockaddr,
) -> MesateeResult<*mut Mesatee> {
    let enclave_info = sanitize_ptr_for_ref(enclave_info_ptr)?;
    if user_id_ptr.is_null() {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    let user_id = ffi::CStr::from_ptr(user_id_ptr)
        .to_str()
        .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
    if user_token_ptr.is_null() {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    let user_token = ffi::CStr::from_ptr(user_id_ptr)
        .to_str()
        .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
    let tms_addr = sockaddr_to_rust(tms_addr_ptr)?;
    let tdfs_addr = sockaddr_to_rust(tdfs_addr_ptr)?;
    let mesatee = Mesatee::new(enclave_info, user_id, user_token, tms_addr, tdfs_addr)?;
    let mesatee_ptr = Box::into_raw(Box::new(mesatee)) as *mut Mesatee;
    Ok(mesatee_ptr)
}

unsafe fn inner_mesatee_context_new2(
    enclave_info_ptr: *mut MesateeEnclaveInfo,
    user_id_ptr: *const c_char,
    user_token_ptr: *const c_char,
    tms_addr_ptr: *const c_char,
    tdfs_addr_ptr: *const c_char,
) -> MesateeResult<*mut Mesatee> {
    let enclave_info = sanitize_ptr_for_ref(enclave_info_ptr)?;
    if user_id_ptr.is_null() {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    let user_id = ffi::CStr::from_ptr(user_id_ptr)
        .to_str()
        .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
    if user_token_ptr.is_null() {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    let user_token = ffi::CStr::from_ptr(user_id_ptr)
        .to_str()
        .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;

    if tms_addr_ptr.is_null() {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    let tms_addr = ffi::CStr::from_ptr(tms_addr_ptr)
        .to_str()
        .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
    if tdfs_addr_ptr.is_null() {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    let tdfs_addr = ffi::CStr::from_ptr(tdfs_addr_ptr)
        .to_str()
        .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
    let tmsaddr = tms_addr
        .parse()
        .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
    let tdfsaddr = tdfs_addr
        .parse()
        .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
    let mesatee = Mesatee::new(enclave_info, user_id, user_token, tmsaddr, tdfsaddr)?;
    let mesatee_ptr = Box::into_raw(Box::new(mesatee)) as *mut Mesatee;
    Ok(mesatee_ptr)
}

unsafe fn sockaddr_to_rust(sockaddr_ptr: *const sockaddr) -> MesateeResult<net::SocketAddr> {
    if sockaddr_ptr.is_null() {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    let nix_sockaddr = SockAddr::from_libc_sockaddr(sockaddr_ptr)
        .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?; // Not a valid libc::sockaddr
    let nix_inet_addr = match nix_sockaddr {
        SockAddr::Inet(addr) => addr,
        _ => return Err(Error::from(ErrorKind::InvalidInputError)), // Not an INET address
    };
    Ok(nix_inet_addr.to_std())
}

#[no_mangle]
unsafe extern "C" fn mesatee_context_free(ctx_ptr: *mut Mesatee) -> c_int {
    check_inner_result!(inner_mesatee_context_free(ctx_ptr), MESATEE_ERROR)
}

unsafe fn inner_mesatee_context_free(ctx_ptr: *mut Mesatee) -> MesateeResult<c_int> {
    let _ = sanitize_ptr_for_mut_ref(ctx_ptr)?;
    let _ = Box::from_raw(ctx_ptr);
    Ok(MESATEE_SUCCESS)
}
