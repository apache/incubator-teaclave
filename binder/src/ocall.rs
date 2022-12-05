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

use sgx_types::error::SgxStatus;
use sgx_types::function::{
    sgx_get_quote_ex, sgx_get_quote_size_ex, sgx_init_quote_ex, sgx_select_att_key_id,
};
use sgx_types::types::*;
use std::ptr;

#[cfg(sgx_sim)]
#[link(name = "sgx_quote_ex_sim")]
#[cfg(not(sgx_sim))]
#[link(name = "sgx_quote_ex")]
extern "C" {
    fn sgx_select_att_key_id(
        p_att_key_id_list: *const u8,
        att_key_idlist_size: u32,
        p_att_key_id: *mut AttKeyId,
    ) -> SgxStatus;

    fn sgx_init_quote_ex(
        p_att_key_id: *const AttKeyId,
        p_qe_target_info: *mut TargetInfo,
        p_pub_key_id_size: *mut usize,
        p_pub_key_id: *mut u8,
    ) -> SgxStatus;

    fn sgx_get_quote_size_ex(p_att_key_id: *const AttKeyId, p_quote_size: *mut u32) -> SgxStatus;

    fn sgx_get_quote_ex(
        p_isv_enclave_report: *const Report,
        p_att_key_id: *const AttKeyId,
        p_qe_report: *mut QeReportInfo,
        p_quote: *mut u8,
        quote_size: u32,
    ) -> SgxStatus;
}

#[no_mangle]
pub extern "C" fn ocall_sgx_init_quote(
    p_att_key_id: *mut AttKeyId,
    p_qe_target_info: *mut TargetInfo,
) -> SgxStatus {
    let ret = unsafe { sgx_select_att_key_id(ptr::null(), 0, p_att_key_id) };

    if ret != SgxStatus::Success {
        return ret;
    }

    // First call to sgx_init_quote_ex to get att_pub_key_id_size
    let mut att_pub_key_id_size = 0usize;
    let ret = unsafe {
        sgx_init_quote_ex(
            p_att_key_id,
            p_qe_target_info,
            &mut att_pub_key_id_size as _,
            ptr::null_mut(),
        )
    };

    if ret != SgxStatus::Success {
        return ret;
    }

    // Second call to sgx_init_quote_ex to get att_pub_key_id
    // At this point, it is unknown what att_pub_key_id is used for.
    let mut att_pub_key_id: Vec<u8> = vec![0u8; att_pub_key_id_size];
    unsafe {
        sgx_init_quote_ex(
            p_att_key_id,
            p_qe_target_info,
            &mut att_pub_key_id_size as _,
            att_pub_key_id.as_mut_ptr(),
        )
    }
}

#[no_mangle]
pub extern "C" fn ocall_sgx_get_quote_size(
    p_att_key_id: *const AttKeyId,
    p_quote_size: *mut u32,
) -> SgxStatus {
    unsafe { sgx_get_quote_size_ex(p_att_key_id as _, p_quote_size) }
}

#[no_mangle]
pub extern "C" fn ocall_sgx_get_quote(
    p_report: *const Report,
    p_att_key_id: *const AttKeyId,
    p_qe_report_info: *mut QeReportInfo,
    p_quote: *mut u8,
    quote_size: u32,
) -> SgxStatus {
    unsafe {
        sgx_get_quote_ex(
            p_report,
            p_att_key_id,
            p_qe_report_info,
            p_quote as _,
            quote_size,
        )
    }
}
