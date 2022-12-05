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
#![allow(non_camel_case_types)]

use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "mesalock_sgx")] {
        pub(crate) use sgx_types::types::c_void;
        pub(crate) use std::ffi::CStr;

        pub(crate) use sgx_types::types::size_t;
        pub(crate) use sgx_types::types::Key128bit;
        pub(crate) use sgx_types::types::Mac128bit;

        pub(crate) use sgx_types::types::{c_char, c_int};
        pub(crate) use sgx_types::types::{int32_t, int64_t};
        pub(crate) use sgx_types::error::OsError as OsErrorNum;
        pub(crate) type OsError = std::result::Result<(), OsErrorNum>;
        pub(crate) use sgx_types::error::OsResult;
        pub(crate) use sgx_trts::error::errno;

        pub(crate) use core::cmp;
    } else {
        pub(crate) use std::ffi::c_void;
        pub(crate) use std::ffi::CStr;

        pub(crate) use libc::size_t;
        pub(crate) type Key128bit = [u8; 16];
        pub(crate) type Mac128bit = [u8; 16];

        pub(crate) use std::os::raw::{c_char, c_int};
        pub(crate) type int32_t = i32;
        pub(crate) type int64_t = i64;
        pub(crate) type os_error_t = int32_t;
        pub(crate) type OsError = std::result::Result<(), os_error_t>;
        pub(crate) type OsResult<T> = std::result::Result<T, os_error_t>;

        pub(crate) fn errno() -> i32 {
            std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
        }

        pub(crate) use std::cmp;
    }
}
