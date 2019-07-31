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

use crate::trait_defs::{Worker, WorkerHelper, WorkerInput};
use mesatee_core::{Error, ErrorKind, Result};
use std::collections::HashMap;

mod demo_func;
mod gbdt;
mod image_resize;
mod kmeans;
mod logistic_reg;
mod online_decrypt;
mod private_join_and_compute;
mod psi;
mod rsa;
mod wasm;

mod mesapy;

pub type HandlerFunc = fn(&mut WorkerHelper, input: WorkerInput) -> Result<String>;

pub struct TrustedWorker {
    dispatcher: HashMap<String, HandlerFunc>,
}

impl TrustedWorker {
    pub fn new() -> Self {
        let mut dispatcher: HashMap<String, HandlerFunc> = HashMap::new();

        dispatcher.insert("echo".to_string(), demo_func::echo);
        dispatcher.insert("bytes_plus_one".to_string(), demo_func::bytes_plus_one);
        dispatcher.insert("echo_file".to_string(), demo_func::echo_file);
        dispatcher.insert(
            "file_bytes_plus_one".to_string(),
            demo_func::file_bytes_plus_one,
        );
        dispatcher.insert("concat".to_string(), demo_func::concat);
        dispatcher.insert("swap_file".to_string(), demo_func::swap_file);
        dispatcher.insert("psi".to_string(), psi::psi);
        dispatcher.insert("wasmi_from_buffer".to_string(), wasm::wasmi_from_buffer);
        dispatcher.insert("mesapy_from_buffer".to_string(), mesapy::mesapy_from_buffer);
        dispatcher.insert(
            "private_join_and_compute".to_string(),
            private_join_and_compute::private_join_and_compute,
        );
        dispatcher.insert("gbdt_predict".to_string(), gbdt::predict);
        dispatcher.insert("image_resize".to_string(), image_resize::process);
        dispatcher.insert("decrypt".to_string(), online_decrypt::decrypt);
        dispatcher.insert("rsa_sign".to_string(), rsa::sign);
        dispatcher.insert("kmeans_cluster".to_string(), kmeans::cluster);
        dispatcher.insert("logistic_reg".to_string(), logistic_reg::cluster);

        TrustedWorker { dispatcher }
    }
}
impl Worker for TrustedWorker {
    fn launch(&mut self) -> Result<()> {
        Ok(())
    }
    fn compute(&mut self, helper: &mut WorkerHelper) -> Result<String> {
        let input = helper.get_input();
        let func_handler = self.dispatcher.get(&input.function_name);
        match func_handler {
            Some(handler) => handler(helper, input),
            None => Err(Error::from(ErrorKind::FunctionNotSupportedError)),
        }
    }
    fn finalize(&mut self) -> Result<()> {
        Ok(())
    }
}
