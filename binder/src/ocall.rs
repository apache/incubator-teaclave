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

use sgx_types::*;
use std::net::TcpStream;
use std::os::unix::io::IntoRawFd;

#[link(name = "sgx_uae_service")]
extern "C" {
    fn sgx_init_quote(
        p_target_info: *mut sgx_target_info_t,
        p_gid: *mut sgx_epid_group_id_t,
    ) -> sgx_status_t;
    fn sgx_calc_quote_size(
        p_sig_rl: *const u8,
        sig_rl_size: u32,
        p_quote_size: *mut u32,
    ) -> sgx_status_t;
    fn sgx_get_quote(
        p_report: *const sgx_report_t,
        quote_type: sgx_quote_sign_type_t,
        p_spid: *const sgx_spid_t,
        p_nonce: *const sgx_quote_nonce_t,
        p_sig_rl: *const u8,
        sig_rl_size: u32,
        p_qe_report: *mut sgx_report_t,
        p_quote: *mut sgx_quote_t,
        quote_size: u32,
    ) -> sgx_status_t;
}

#[no_mangle]
pub extern "C" fn ocall_sgx_get_ias_socket() -> i32 {
    let ias_addr = "api.trustedservices.intel.com:443";
    match TcpStream::connect(ias_addr) {
        Ok(socket) => socket.into_raw_fd(),
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn ocall_sgx_init_quote(
    p_target_info: *mut sgx_target_info_t,
    p_gid: *mut sgx_epid_group_id_t,
) -> sgx_status_t {
    unsafe { sgx_init_quote(p_target_info, p_gid) }
}

#[no_mangle]
pub extern "C" fn ocall_sgx_calc_quote_size(
    p_sig_rl: *const u8,
    sig_rl_size: u32,
    p_quote_size: *mut u32,
) -> sgx_status_t {
    unsafe { sgx_calc_quote_size(p_sig_rl, sig_rl_size, p_quote_size) }
}

#[no_mangle]
pub extern "C" fn ocall_sgx_get_quote(
    p_report: *const sgx_report_t,
    quote_type: sgx_quote_sign_type_t,
    p_spid: *const sgx_spid_t,
    p_nonce: *const sgx_quote_nonce_t,
    p_sig_rl: *const u8,
    sig_rl_size: u32,
    p_qe_report: *mut sgx_report_t,
    p_quote: *mut sgx_quote_t,
    quote_size: u32,
) -> sgx_status_t {
    unsafe {
        sgx_get_quote(
            p_report,
            quote_type,
            p_spid,
            p_nonce,
            p_sig_rl,
            sig_rl_size,
            p_qe_report,
            p_quote,
            quote_size,
        )
    }
}
