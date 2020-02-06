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

// ATTN: Must bring `use ipc::IpcReceiver` into scope when use!!
#[macro_export]
macro_rules! register_ecall_handler {
    ( type $cmd_type: ty, $( ($cmd: path, $arg: ty, $ret: ty), )* ) =>
    {
        fn ecall_ipc_lib_dispatcher(cmd: u32, input: &[u8]) -> anyhow::Result<Vec<u8>> {
            let cmd = <$cmd_type>::from(cmd);
            match cmd {
                $(
                    $cmd => dispatch_helper::<$arg, $ret>(input),
                )*
                _ => anyhow::bail!("ECallCommandNotRegistered"),
            }
        }
        use teaclave_ipc::IpcService;
        use teaclave_ipc::IpcReceiver;

        // Declear a local trait, the [handle_ecall] attribute macro
        // will help implement this trait and call user defined function.
        trait HandleRequest<V> {
            fn handle(&self) -> teaclave_types::TeeServiceResult<V>;
        }

        struct ServeInstance<U, V>
        where
            U: HandleRequest<V> + for<'de> serde::Deserialize<'de>,
            V: serde::Serialize,
        {
            u: std::marker::PhantomData<U>,
            v: std::marker::PhantomData<V>,
        }

        impl<U, V> ServeInstance<U, V>
        where
            U: HandleRequest<V> + for<'de> serde::Deserialize<'de>,
            V: serde::Serialize,
        {
            fn new() -> ServeInstance<U, V> {
                ServeInstance {
                    u: std::marker::PhantomData,
                    v: std::marker::PhantomData,
                }
            }
        }

        impl<U, V> IpcService<U, V> for ServeInstance<U, V>
        where
            U: HandleRequest<V> + for<'de> serde::Deserialize<'de>,
            V: serde::Serialize,
        {
            fn handle_invoke(&self, input: U) -> teaclave_types::TeeServiceResult<V> {
                log::debug!("handle_invoke");
                input.handle()
            }
        }

        fn dispatch_helper<U, V>(input: &[u8]) -> anyhow::Result<Vec<u8>>
        where
            U: HandleRequest<V> + for<'de> serde::Deserialize<'de>,
            V: serde::Serialize,
        {
            let instance = ServeInstance::<U, V>::new();
            teaclave_ipc::channel::ECallReceiver::dispatch(input, instance)
        }

        /// The actual ecall function defined in .edl.
        #[cfg(not(feature="enclave_unit_test"))]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        #[no_mangle]
        pub extern "C" fn ecall_ipc_entry_point(
            cmd: u32,
            in_buf: *const u8,
            in_len: usize,
            out_buf: *mut u8,
            out_max: usize,
            out_len: &mut usize,
        ) -> teaclave_types::EnclaveStatus {
            // The last argument could be either * mut usize, or &mut usize
            let input_buf: &[u8] = unsafe { std::slice::from_raw_parts(in_buf, in_len) };

            trace!("tee receive cmd: {:x}, input_buf = {:?}", cmd, input_buf);

            let inner_vec = unsafe {
                match ecall_ipc_lib_dispatcher(cmd, input_buf) {
                    Ok(out) => out,
                    Err(e) => {
                        error!("tee execute cmd: {:x}, error: {}", cmd, e);
                        return teaclave_types::EnclaveStatus(1);
                    }
                }
            };

            let inner_len = inner_vec.len();

            // ATTN: We should always set the out_len, no matter whether it is within the buffer range.
            *out_len = inner_len;

            if inner_len > out_max {
                debug!("tee before copy out_buf check: out_max={:x} < inner={:x}", out_max, inner_len);
                return teaclave_types::EnclaveStatus(0x0000_000c);
            }

            // The following lines use a trick of "constructing a mutable slice
            // in place" using slice::from_raw_parts_mut You can always use
            // ptr::copy_nonoverlapping to copy a buffer to the output pointer
            // (see the above crypto sample)
            unsafe {
                std::ptr::copy_nonoverlapping(inner_vec.as_ptr(), out_buf, inner_len);
            }

            // out_len would be used in `set_len` in the untrusted app
            // so out_len cannot be larger than out_max. Additional checks are **required**.
            teaclave_types::EnclaveStatus::default()
        }
    }
}
