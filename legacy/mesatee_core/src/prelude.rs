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

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(feature = "mesalock_sgx", feature = "ipc"))] {
        // preludes provided for SGX enclave.
        pub use crate::register_ecall_handler;
        pub use crate::rpc::{RpcServer, RpcClient, EnclaveService};
        pub use crate::rpc::sgx::{Pipe, PipeConfig};
        pub use crate::ipc::{IpcSender, IpcService, IpcReceiver};
        pub use crate::ipc::protos::ECallCommand;
        pub use crate::ipc::protos::ecall::{
            StartServiceInput,
            StartServiceOutput,
            InitEnclaveInput,
            InitEnclaveOutput,
            FinalizeEnclaveInput,
            FinalizeEnclaveOutput,
            ServeConnectionInput,
            ServeConnectionOutput,
        };
        pub use crate::ipc_attribute::handle_ecall;
    } else if #[cfg(all(not(feature = "mesalock_sgx"), feature = "ipc"))] {
        // preludes provided for sgx_app
        pub use crate::ipc::protos::ECallCommand;
        pub use crate::ipc::protos::ecall::{
            StartServiceInput,
            StartServiceOutput,
            ServeConnectionInput,
            ServeConnectionOutput,
        };

    } else if #[cfg(all(not(feature = "mesalock_sgx"), not(feature = "ipc")))] {
        // preludes provided for unix app
    } else {
        // This should not happen!
        #[deprecated(
            since = "0.0.0",
            note = "mesatee_core is being used in wrong feature combination!"
        )]
        fn something_bad_happens() {
            stop_the_compilation // Cannot compile!
        }
    }
}
