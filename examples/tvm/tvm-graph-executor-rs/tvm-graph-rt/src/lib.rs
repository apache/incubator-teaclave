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

//! This crate is an implementation of the TVM runtime for modules compiled with `--system-lib`.
//! It's mainly useful for compiling to WebAssembly and SGX,
//! but also native if you prefer Rust to C++.
//!
//! For TVM graphs, the entrypoint to this crate is `runtime::GraphExecutor`.
//! Single-function modules are used via the `packed_func!` macro after obtaining
//! the function from `runtime::SystemLibModule`
//!
//! The main entrypoints to this crate are `GraphExecutor`
//! For examples of use, please refer to the multi-file tests in the `tests` directory.
#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
#[macro_use]
extern crate sgx_tstd as std;

extern crate tvm_macros;
extern crate tvm_sys;

// Re-export the import_module macro.
pub use tvm_macros::import_module;

// Re-export the called pack macro, eventually remove as its not a very good
// abstraction.
pub use tvm_sys::call_packed;

use lazy_static::lazy_static;

mod allocator;
mod array;
pub mod errors;
mod graph;
mod module;
//mod threading;
mod workspace;
use tvm_sys::ffi::TVMParallelGroupEnv;

pub(crate) type FTVMParallelLambda =
    extern "C" fn(task_id: usize, penv: *const TVMParallelGroupEnv, cdata: *const c_void) -> i32;
use std::os::raw::{c_int, c_void};
#[no_mangle]
pub extern "C" fn TVMBackendParallelLaunch(
    cb: FTVMParallelLambda,
    cdata: *const c_void,
    num_task: usize,
) -> c_int {
    println!("TVMBackendParallelLaunch: {:?}", num_task);
    let penv = TVMParallelGroupEnv {
        sync_handle: std::ptr::null_mut(),
        num_task: 1,
    };
    let r = cb(0, &penv as *const _, cdata);
    println!("cb result: {:?}", r);
    r
}

pub use tvm_sys::{
    errors::*,
    ffi::{self, DLTensor},
    packed_func::{self, *},
    ArgValue, RetValue,
};

//pub use self::{array::*, errors::*, graph::*, module::*, threading::*, workspace::*};
pub use self::{array::*, errors::*, graph::*, module::*, workspace::*};

lazy_static! {
    static ref LAST_ERROR: std::sync::SgxRwLock<Option<&'static std::ffi::CStr>> =
        std::sync::SgxRwLock::new(None);
}

#[no_mangle]
pub unsafe extern "C" fn TVMAPISetLastError(cmsg: *const i8) {
    println!("[-] TVMAPISetLastError: {:?}", cmsg);
    *LAST_ERROR.write().unwrap() = Some(std::ffi::CStr::from_ptr(cmsg));
}

#[no_mangle]
pub extern "C" fn TVMGetLastError() -> *const std::os::raw::c_char {
    match *LAST_ERROR.read().unwrap() {
        Some(err) => err.as_ptr(),
        None => std::ptr::null(),
    }
}
