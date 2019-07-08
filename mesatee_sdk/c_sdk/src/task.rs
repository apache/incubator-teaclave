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
use libc::{c_char, c_uchar};
use mesatee_sdk::{Mesatee, MesateeTask};
use std::{ffi, ptr, slice, str};

impl OpaquePointerType for MesateeTask {}

#[no_mangle]
unsafe extern "C" fn mesatee_create_task(
    ctx_ptr: *mut Mesatee,
    func_name_ptr: *const c_char,
) -> *mut MesateeTask {
    check_inner_result!(
        inner_mesatee_create_task(ctx_ptr, func_name_ptr),
        ptr::null_mut()
    )
}

unsafe fn inner_mesatee_create_task(
    ctx_ptr: *mut Mesatee,
    func_name_ptr: *const c_char,
) -> MesateeResult<*mut MesateeTask> {
    let ctx = sanitize_ptr_for_ref(ctx_ptr)?;
    if func_name_ptr.is_null() {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }
    let func_name = ffi::CStr::from_ptr(func_name_ptr)
        .to_str()
        .map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
    let task = ctx.create_task(func_name)?;
    let task_ptr = Box::into_raw(Box::new(task)) as *mut MesateeTask;
    Ok(task_ptr)
}

#[no_mangle]
unsafe extern "C" fn mesatee_task_free(mesatee_task_ptr: *mut MesateeTask) -> c_int {
    check_inner_result!(inner_mesatee_task_free(mesatee_task_ptr), MESATEE_ERROR)
}

unsafe fn inner_mesatee_task_free(mesatee_task_ptr: *mut MesateeTask) -> MesateeResult<c_int> {
    let _ = sanitize_ptr_for_mut_ref(mesatee_task_ptr)?;
    let _ = Box::from_raw(mesatee_task_ptr);
    Ok(MESATEE_SUCCESS)
}

#[no_mangle]
unsafe extern "C" fn mesatee_task_invoke_with_payload(
    mesatee_task_ptr: *mut MesateeTask,
    payload_buf_ptr: *const c_uchar,
    payload_buf_len: c_int,
    result_buf_ptr: *mut c_uchar,
    result_buf_len: c_int,
) -> c_int {
    check_inner_result!(
        inner_mesatee_task_invoke_with_payload(
            mesatee_task_ptr,
            payload_buf_ptr,
            payload_buf_len,
            result_buf_ptr,
            result_buf_len
        ),
        MESATEE_ERROR
    )
}

unsafe fn inner_mesatee_task_invoke_with_payload(
    mesatee_task_ptr: *mut MesateeTask,
    payload_buf_ptr: *const c_uchar,
    payload_buf_len: c_int,
    output_buf_ptr: *mut c_uchar,
    output_buf_len: c_int,
) -> MesateeResult<c_int> {
    let task = sanitize_ptr_for_mut_ref(mesatee_task_ptr)?;

    if payload_buf_ptr.is_null() || output_buf_ptr.is_null() {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }

    let payload_buf: &[u8] = slice::from_raw_parts(payload_buf_ptr, payload_buf_len as usize);
    let output_buf: &mut [u8] = slice::from_raw_parts_mut(output_buf_ptr, output_buf_len as usize);

    let payload_str = str::from_utf8_unchecked(&payload_buf);
    let result = task.invoke_with_payload(&payload_str)?;
    let result_bytes = result.as_bytes();

    if result_bytes.len() >= output_buf.len() {
        // Handle off-by-one
        output_buf.copy_from_slice(&result_bytes[..output_buf.len()]);
        output_buf[output_buf.len() - 1] = 0;
    } else {
        output_buf[0..result_bytes.len()].copy_from_slice(&result_bytes[..])
    }

    Ok(MESATEE_SUCCESS)
}
