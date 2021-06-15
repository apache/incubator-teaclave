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

use std::ffi::{c_void, CString};

use teaclave_types::{FunctionArguments, FunctionRuntime, TeaclaveExecutor};

const DEFAULT_HEAP_SIZE: u32 = 8092;
const DEFAULT_STACK_SIZE: u32 = 8092;
const DEFAULT_ERROR_BUF_SIZE: usize = 128;

extern "C" {

    fn wasm_runtime_init() -> bool;

    fn wasm_runtime_load(
        buf: *const u8,
        size: u32,
        error_buf: *mut u8,
        error_buf_size: u32,
    ) -> *const c_void;

    fn wasm_runtime_instantiate(
        module: *const c_void,
        stack_size: u32,
        heap_size: u32,
        error_buf: *mut u8,
        error_buf_size: u32,
    ) -> *const c_void;

    fn wasm_runtime_lookup_function(
        module_inst: *const c_void,
        name: *const i8,
        signature: *const u8,
    ) -> *const c_void;

    fn wasm_runtime_create_exec_env(module_inst: *const c_void, stack_size: u32) -> *const c_void;

    fn wasm_runtime_call_wasm(
        exec_env: *const c_void,
        function: *const c_void,
        argc: u32,
        argv: *const u32,
    ) -> bool;

    fn wasm_runtime_module_dup_data(module_inst: *const c_void, src: *const u8, size: u32) -> u32;

    fn wasm_runtime_module_free(module_inst: *const c_void, ptr: u32);
}

#[derive(Default)]
pub struct WAMicroRuntime;

impl TeaclaveExecutor for WAMicroRuntime {
    fn execute(
        &self,
        _name: String,
        arguments: FunctionArguments,
        payload: Vec<u8>,
        runtime: FunctionRuntime,
    ) -> anyhow::Result<String> {
        let wa_argv: Vec<_> = arguments.into_vec();

        let mut error_buf = [0u8; DEFAULT_ERROR_BUF_SIZE];
        let entry_name = CString::new("entrypoint").expect("CString::new failed");

        set_thread_context(Context::new(runtime))?;

        unsafe { wasm_runtime_init() };
        let module = unsafe {
            wasm_runtime_load(
                payload.as_ptr(),
                payload.len() as u32,
                error_buf.as_mut_ptr(),
                error_buf.len() as u32,
            )
        };

        assert!((module as usize) != 0);

        error_buf = [0u8; DEFAULT_ERROR_BUF_SIZE];
        let module_instance = unsafe {
            wasm_runtime_instantiate(
                module,
                DEFAULT_STACK_SIZE,
                DEFAULT_HEAP_SIZE,
                error_buf.as_mut_ptr(),
                error_buf.len() as u32,
            )
        };
        assert!((module_instance as usize) != 0);

        let entry_func = unsafe {
            wasm_runtime_lookup_function(module_instance, entry_name.as_ptr(), std::ptr::null())
        };
        assert!((entry_func as usize) != 0);

        let exec_env = unsafe { wasm_runtime_create_exec_env(module_instance, DEFAULT_STACK_SIZE) };
        assert!((exec_env as usize) != 0);

        // prepare the arguments
        // for best compatibility with Teaclave, the function signature is `int entrypoint(int argc, char* argv[])`
        let cstr_argv: Vec<_> = wa_argv
            .iter()
            .map(|arg| CString::new(arg.as_str()).unwrap())
            .collect();
        let wasm_argc = 2;
        let p_argv: Vec<u32> = cstr_argv
            .iter() // do NOT into_iter()
            .map(|arg| unsafe {
                wasm_runtime_module_dup_data(
                    module_instance,
                    arg.as_ptr() as *const u8,
                    arg.to_bytes_with_nul().len() as u32,
                )
            })
            .collect();
        let func_argv = unsafe {
            wasm_runtime_module_dup_data(
                module_instance,
                p_argv.as_ptr() as *const u8,
                (p_argv.len() * 4) as u32,
            )
        };
        let wasm_argv: [u32; 2] = [p_argv.len() as u32, func_argv];

        if unsafe { wasm_runtime_call_wasm(exec_env, entry_func, wasm_argc, wasm_argv.as_ptr()) } {
            reset_thread_context()?;
            log::debug!(
                "IN WAMicroRuntime::execute after `wasm_runtime_call_wasm`, {:?}",
                wasm_argv[0]
            );
            // clean WAMR allocated memory
            let _ = p_argv
                .iter()
                .map(|addr| unsafe { wasm_runtime_module_free(module_instance, *addr) });
            unsafe { wasm_runtime_module_free(module_instance, func_argv) };
            Ok(wasm_argv[0].to_string())
        } else {
            Ok("WAMR Error".to_string())
        }
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use std::collections::HashMap;
    use teaclave_runtime::*;
    use teaclave_test_utils::*;
    use teaclave_types::*;

    pub fn run_tests() -> bool {
        run_tests!(test_wamr,)
    }

    fn test_wamr() {
        let mut args = HashMap::new();

        args.insert("adder 1".to_string(), "3".to_string());
        args.insert("adder 2".to_string(), "4".to_string());
        let args = FunctionArguments::from(args);

        let wa_payload =
            include_bytes!("../../examples/python/wasm_simple_add_payload/simple_add.wasm");

        let wa_payload = wa_payload.to_vec();

        let input_files = StagedFiles::default();
        let output_files = StagedFiles::default();

        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));

        let function = WAMicroRuntime::default();
        let summary = function
            .execute("".to_string(), args, wa_payload, runtime)
            .unwrap();
        log::debug!("IN TEST test_wamr: AFTER execution, summary: {:?}", summary);

        assert_eq!(summary, "7");
    }
}
