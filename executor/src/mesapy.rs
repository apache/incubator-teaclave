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

use std::prelude::v1::*;

use crate::context::reset_thread_context;
use crate::context::set_thread_context;
use crate::context::Context;

use std::ffi::CString;

use teaclave_types::{FunctionArguments, FunctionRuntime, TeaclaveExecutor};

const MAXPYBUFLEN: usize = 20480;
const MESAPY_ERROR_BUFFER_TOO_SHORT: i64 = -1i64;
const MESAPY_EXEC_ERROR: i64 = -2i64;

extern "C" {
    fn mesapy_exec(
        input: *const u8,
        argc: usize,
        argv: *const *const sgx_types::c_char,
        output: *mut u8,
        buflen: u64,
    ) -> i64;
}

#[derive(Default)]
pub struct MesaPy;

impl TeaclaveExecutor for MesaPy {
    fn execute(
        &self,
        _name: String,
        arguments: FunctionArguments,
        mut payload: Vec<u8>,
        runtime: FunctionRuntime,
    ) -> anyhow::Result<String> {
        let py_argv = arguments.into_vec();
        let cstr_argv: Vec<_> = py_argv
            .iter()
            .map(|arg| CString::new(arg.as_str()).unwrap())
            .collect();

        payload.push(0u8);

        let mut p_argv: Vec<_> = cstr_argv
            .iter() // do NOT into_iter()
            .map(|arg| arg.as_ptr())
            .collect();

        p_argv.push(std::ptr::null());

        let mut py_result = [0u8; MAXPYBUFLEN];

        set_thread_context(Context::new(runtime))?;

        let result = unsafe {
            mesapy_exec(
                payload.as_ptr(),
                p_argv.len() - 1,
                p_argv.as_ptr(),
                &mut py_result as *mut _ as *mut u8,
                MAXPYBUFLEN as u64,
            )
        };

        reset_thread_context()?;
        match result {
            MESAPY_ERROR_BUFFER_TOO_SHORT => Ok("MESAPY_ERROR_BUFFER_TOO_SHORT".to_string()),
            MESAPY_EXEC_ERROR => Ok("MESAPY_EXEC_ERROR".to_string()),
            len => {
                let r: Vec<u8> = py_result.iter().take(len as usize).copied().collect();
                let payload = String::from_utf8(r)?;
                Ok(payload)
            }
        }
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use teaclave_crypto::*;
    use teaclave_runtime::*;
    use teaclave_test_utils::*;
    use teaclave_types::*;

    pub fn run_tests() -> bool {
        run_tests!(test_mesapy,)
    }

    fn test_mesapy() {
        let py_args = FunctionArguments::default();
        let py_payload = r#"
def entrypoint(argv):
    in_file_id = "in_f1"
    out_file_id = "out_f1"

    # open input via built-in teaclave_open
    with teaclave_open(in_file_id, "rb") as f:
        line = f.readline()
        assert line == "Hello\n"

    # open input via teaclave module
    from teaclave import open
    with open(in_file_id, "rb") as f:
        line = f.readline()
        assert line == "Hello\n"

    # open invalid input
    try:
        teaclave_open("invalid_key", "rb")
    except RuntimeError as e:
        assert e.message == "fileio_init: teaclave_ffi_error"

    # open invalid option
    try:
        teaclave_open(in_file_id, "r")
    except RuntimeError as e:
        assert e.message == "Teaclave Not Supported"

    # write valid output
    with teaclave_open(out_file_id, "wb") as f:
        f.write("This message is from Mesapy!")

    # open invalid output
    try:
        teaclave_open("invalid_key", "wb")
    except RuntimeError as e:
        assert e.message == "fileio_init: teaclave_ffi_error"

    # open invalid option
    try:
        teaclave_open(out_file_id, "w")
    except RuntimeError as e:
        assert e.message == "Teaclave Not Supported"

    return
"#;

        let input = "fixtures/functions/mesapy/input.txt";
        let output = "fixtures/functions/mesapy/output.txt";

        let input_info =
            StagedFileInfo::new(input, TeaclaveFile128Key::random(), FileAuthTag::mock());
        let output_info =
            StagedFileInfo::new(output, TeaclaveFile128Key::random(), FileAuthTag::mock());

        let input_files = StagedFiles::new(hashmap!("in_f1" => input_info));
        let output_files = StagedFiles::new(hashmap!("out_f1" => output_info));

        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));

        let function = MesaPy::default();
        let summary = function
            .execute(
                "".to_string(),
                py_args,
                py_payload.as_bytes().to_vec(),
                runtime,
            )
            .unwrap();
        assert_eq!(summary, "");
    }
}
