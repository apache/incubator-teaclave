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

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use anyhow;
use itertools::Itertools;

use crate::function::TeaclaveFunction;
use crate::runtime::TeaclaveRuntime;
use teaclave_types::FunctionArguments;

use crate::function::context::reset_thread_context;
use crate::function::context::set_thread_context;
use crate::function::context::Context;
use std::ffi::CString;
use std::format;

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
pub struct Mesapy;

impl TeaclaveFunction for Mesapy {
    fn execute(
        &self,
        runtime: Box<dyn TeaclaveRuntime + Send + Sync>,
        args: FunctionArguments,
    ) -> anyhow::Result<String> {
        let script = args.get("py_payload")?.as_str();
        let py_args = args.get("py_args")?.as_str();
        let py_args: FunctionArguments = serde_json::from_str(py_args)?;
        let py_argv = py_args.into_vec();
        let cstr_argv: Vec<_> = py_argv
            .iter()
            .map(|arg| CString::new(arg.as_str()).unwrap())
            .collect();

        let mut script_bytes = script.to_owned().into_bytes();
        script_bytes.push(0u8);

        let mut p_argv: Vec<_> = cstr_argv
            .iter() // do NOT into_iter()
            .map(|arg| arg.as_ptr())
            .collect();

        p_argv.push(std::ptr::null());

        let mut py_result = [0u8; MAXPYBUFLEN];

        set_thread_context(Context::new(runtime))?;

        let result = unsafe {
            mesapy_exec(
                script_bytes.as_ptr(),
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
                let payload = format!("marshal.loads(b\"\\x{:02X}\")", r.iter().format("\\x"));
                Ok(payload)
            }
        }
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use teaclave_test_utils::*;

    use crate::function::TeaclaveFunction;
    use crate::runtime::RawIoRuntime;
    use teaclave_types::hashmap;
    use teaclave_types::FunctionArguments;
    use teaclave_types::StagedFileInfo;
    use teaclave_types::StagedFiles;
    use teaclave_types::TeaclaveFile128Key;

    pub fn run_tests() -> bool {
        run_tests!(test_mesapy,)
    }

    fn test_mesapy() {
        let py_args = FunctionArguments::new(hashmap!("--name" => "Teaclave"));
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
"#;

        let input = "fixtures/functions/mesapy/input.txt";
        let output = "fixtures/functions/mesapy/output.txt";

        let input_info = StagedFileInfo::new(input, TeaclaveFile128Key::random());

        let output_info = StagedFileInfo::new(output, TeaclaveFile128Key::random());

        let input_files = StagedFiles {
            entries: hashmap!("in_f1".to_string() => input_info),
        };

        let output_files = StagedFiles {
            entries: hashmap!("out_f1".to_string() => output_info),
        };
        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));

        let func_args = FunctionArguments::new(hashmap!(
                "py_payload" => py_payload.to_string(),
                "py_args" => serde_json::to_string(&py_args).unwrap()
        ));

        let function = Mesapy;
        let summary = function.execute(runtime, func_args).unwrap();
        assert_eq!(summary, "marshal.loads(b\"\\x4E\")");
    }
}
