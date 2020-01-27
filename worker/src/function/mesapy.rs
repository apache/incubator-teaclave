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

use crate::function::TeaclaveFunction;
use crate::runtime::TeaclaveRuntime;
use teaclave_types::TeaclaveFunctionArguments;

/* TODO: export wrapped io stream handle to mesapy-sgx
extern "C"
t_open(context, file_identifier) -> handle  () {
    runtime = c_to_rust(context);  // thread_local
    runtime.open(file_identifier);
}
t_read(context, handle, buf);
t_write(context, handle, buf);
t_close(context, handle);
*/

#[derive(Default)]
pub struct Mesapy;

impl TeaclaveFunction for Mesapy {
    fn execute(
        &self,
        _runtime: Box<dyn TeaclaveRuntime + Send + Sync>,
        _args: TeaclaveFunctionArguments,
    ) -> anyhow::Result<String> {
        // TODO:
        // args.get("py_payload")
        // args.get("py_args")
        // mesapy_exec();
        unimplemented!()
    }
}
