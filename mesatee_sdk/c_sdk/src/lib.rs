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

use libc::c_int;
use log::error;
use mesatee_core::{Error, ErrorKind};

pub(self) use auditor_set::MesateeAuditorSet;
pub(self) type MesateeResult<T> = Result<T, Error>;

#[repr(C)]
pub(self) enum MesateeRetcode {
    Error = 0,
    Success = 1,
}

pub(self) const MESATEE_ERROR: c_int = MesateeRetcode::Error as c_int;
pub(self) const MESATEE_SUCCESS: c_int = MesateeRetcode::Success as c_int;

#[doc(hidden)]
#[macro_export]
macro_rules! check_inner_result {
    ($inner:expr, $err_ret:expr) => {{
        use std::panic;
        match panic::catch_unwind(panic::AssertUnwindSafe(|| $inner))
            .unwrap_or_else(|_| Err(Error::from(ErrorKind::FFIError)))
        {
            Ok(r) => r,
            Err(e) => {
                error!("MesaTEE SDK panicked: {:?}", e);
                $err_ret
            }
        }
    }};
}

pub(self) trait OpaquePointerType {}

pub(self) fn sanitize_const_ptr_for_ref<'a, T>(ptr: *const T) -> MesateeResult<&'a T>
where
    T: OpaquePointerType,
{
    let ptr = ptr as *mut T;
    sanitize_ptr_for_mut_ref(ptr).map(|r| r as &'a T)
}
pub(self) fn sanitize_ptr_for_ref<'a, T>(ptr: *mut T) -> MesateeResult<&'a T>
where
    T: OpaquePointerType,
{
    sanitize_ptr_for_mut_ref(ptr).map(|r| r as &'a T)
}

pub(self) fn sanitize_ptr_for_mut_ref<'a, T>(ptr: *mut T) -> MesateeResult<&'a mut T>
where
    T: OpaquePointerType,
{
    if !ptr.is_null() {
        let obj_ref: &mut T = unsafe { &mut *ptr };
        Ok(obj_ref)
    } else {
        Err(Error::from(ErrorKind::FFIError))
    }
}

mod auditor_set;
mod context;
mod enclave_info;
mod task;
