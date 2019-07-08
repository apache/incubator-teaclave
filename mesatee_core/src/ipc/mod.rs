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
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

// Intra-Process-Communication
// Developer should split a process into two partitions, App and TEE.

// Caller of an IPC function
// Generic U: ArgmentsInfo type
// Generic V: ReturnInfo type
pub trait IpcSender {
    fn invoke<U, V>(&mut self, cmd: u32, input: U) -> Result<V>
    where
        U: Serialize,
        V: DeserializeOwned;
}

// Service Instance of an IPC function
// Generic U: ArgmentsInfo type
// Generic V: ReturnInfo type
pub trait IpcService<U, V>
where
    U: DeserializeOwned,
    V: Serialize,
{
    fn handle_invoke(&self, input: U) -> Result<V>;
}

// Callee of an IPC function
// Generic U: ArgmentsInfo type
// Generic V: ReturnInfo type
// Generic X: Service Instance that serves of <U, V>
// Dispatch a received input to a specific service instance.
pub trait IpcReceiver {
    fn dispatch<U, V, X>(input: &[u8], x: X) -> Result<Vec<u8>>
    where
        U: DeserializeOwned,
        V: Serialize,
        X: IpcService<U, V>;
}

pub mod channel;
pub mod macros;
pub mod protos;
