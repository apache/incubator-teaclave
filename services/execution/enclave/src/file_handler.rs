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

use anyhow::Result;
#[cfg(feature = "mesalock_sgx")]
use sgx_types::error::SgxStatus;
use teaclave_types::FileAgentRequest;

#[cfg(feature = "mesalock_sgx")]
extern "C" {
    fn ocall_handle_file_request(p_retval: *mut u32, in_buf: *const u8, in_len: u32) -> SgxStatus;
}

#[cfg(feature = "mesalock_sgx")]
pub(crate) fn handle_file_request(request: FileAgentRequest) -> Result<()> {
    let mut rt: u32 = 2;
    let bytes = serde_json::to_vec(&request)?;
    let buf_len = bytes.len();
    let res =
        unsafe { ocall_handle_file_request(&mut rt as _, bytes.as_ptr() as _, buf_len as u32) };
    anyhow::ensure!(res == SgxStatus::Success, "ocall sgx_error = {:?}", res);
    anyhow::ensure!(rt == 0, "ocall error = {:?}", rt);
    Ok(())
}

#[cfg(not(feature = "mesalock_sgx"))]
pub(crate) fn handle_file_request(request: FileAgentRequest) -> Result<()> {
    let bytes = serde_json::to_vec(&request)?;
    teaclave_file_agent::handle_file_request(&bytes)
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::vec;
    use teaclave_types::*;
    use url::Url;

    pub fn test_handle_file_request() {
        let test_install_dir = env!("TEACLAVE_TEST_INSTALL_DIR");
        let fixture_dir = format!(
            "file:///{}/fixtures/functions/mesapy/input.txt",
            test_install_dir
        );
        let url = Url::parse(&fixture_dir).unwrap();
        let dest = PathBuf::from("/tmp/execution_input_test.txt");

        let info = HandleFileInfo::new(&dest, &url);
        let request =
            FileAgentRequest::new(HandleFileCommand::Download, vec![info], "/tmp/fusion_data");

        handle_file_request(request).unwrap();
        std::untrusted::fs::remove_file(&dest).unwrap();
    }
}
