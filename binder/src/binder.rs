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

use sgx_types::function::sgx_destroy_enclave;
use sgx_urts::enclave::SgxEnclave;

use log::{debug, error};
use serde::{Deserialize, Serialize};

use crate::error::TeeBinderError;
use crate::ipc::ECallChannel;
use crate::ipc::IpcSender;
use crate::proto::{
    ECallCommand, FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
};
use teaclave_types::TeeServiceResult;

const ENCLAVE_FILE_SUFFIX: &str = "_enclave.signed.so";

pub struct TeeBinder {
    enclave: SgxEnclave,
}

impl TeeBinder {
    pub fn new(name: &str) -> Result<TeeBinder, TeeBinderError> {
        let enclave = if cfg!(production) {
            create_sgx_enclave(name, false)?
        } else {
            create_sgx_enclave(name, true)?
        };
        debug!("EnclaveID: {}", enclave.eid());

        let tee = TeeBinder { enclave };

        let _ = tee.invoke::<InitEnclaveInput, TeeServiceResult<InitEnclaveOutput>>(
            ECallCommand::InitEnclave,
            InitEnclaveInput,
        )?;

        Ok(tee)
    }

    pub fn invoke<U, V>(&self, command: ECallCommand, input: U) -> Result<V, TeeBinderError>
    where
        U: Serialize,
        V: for<'de> Deserialize<'de>,
    {
        let mut channel = ECallChannel::new(self.enclave.eid());
        channel
            .invoke::<U, V>(command.into(), input)
            .map_err(TeeBinderError::IpcError)
    }

    pub fn finalize(&self) {
        match self.invoke::<FinalizeEnclaveInput, TeeServiceResult<FinalizeEnclaveOutput>>(
            ECallCommand::FinalizeEnclave,
            FinalizeEnclaveInput,
        ) {
            Ok(_) => {}
            Err(e) => error!("{:?}", e),
        }
    }

    /// # Safety
    /// Force to destroy current enclave.
    pub unsafe fn destroy(&self) {
        let _ = sgx_destroy_enclave(self.enclave.eid());
    }

    #[cfg(feature = "app_unit_test")]
    pub fn run_app_tests(&self) -> bool {
        crate::ipc::app::tests::run_tests(self.enclave.eid())
    }
}

impl Drop for TeeBinder {
    fn drop(&mut self) {
        debug!("Dropping TeeBinder, start finalize().");
        self.finalize();
    }
}

fn create_sgx_enclave(
    enclave_name: &str,
    debug_launch: bool,
) -> Result<SgxEnclave, TeeBinderError> {
    let enclave_file = format!("{}{}", enclave_name, ENCLAVE_FILE_SUFFIX);

    let enclave =
        SgxEnclave::create(enclave_file, debug_launch).map_err(TeeBinderError::SgxError)?;

    Ok(enclave)
}
