// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Insert std prelude in the top for the sgx feature

use cfg_if::cfg_if;
use serde_derive::{Deserialize, Serialize};

// Use target specific definitions here
cfg_if! {
    if #[cfg(feature = "mesalock_sgx")]  {
        use sgx_types::c_int;
    } else {
        use std::os::raw::c_int;
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct InitEnclaveInput;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct InitEnclaveOutput;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct FinalizeEnclaveInput;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct FinalizeEnclaveOutput;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct RunFunctionalTestInput;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RunFunctionalTestOutput {
    pub failed_count: usize,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ServeConnectionInput {
    pub socket_fd: c_int,
    pub port: u16,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct ServeConnectionOutput;

impl RunFunctionalTestOutput {
    pub fn new(failed_count: usize) -> RunFunctionalTestOutput {
        RunFunctionalTestOutput { failed_count }
    }
}

impl ServeConnectionInput {
    pub fn new(socket_fd: c_int, port: u16) -> ServeConnectionInput {
        ServeConnectionInput { socket_fd, port }
    }
}
