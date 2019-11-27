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

use sgx_types::*;
use sgx_urts::SgxEnclave;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::mem;
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::Serialize;

use mesatee_core::ipc::channel::ECallChannel;
use mesatee_core::ipc::protos::ecall::{FinalizeEnclaveInput, FinalizeEnclaveOutput};
use mesatee_core::ipc::protos::ecall::{InitEnclaveInput, InitEnclaveOutput};
use mesatee_core::ipc::protos::ECallCommand;
use mesatee_core::ipc::IpcSender;
use mesatee_core::{utils::sgx_launch_check, Error, ErrorKind, Result};

static ENCLAVE_FILE_SUFFIX: &str = "enclave.signed.so";
static ENCLAVE_TOKEN_SUFFIX: &str = "enclave.token";

const TOKEN_LEN: usize = mem::size_of::<sgx_launch_token_t>();

pub use crate::ocall::ocall_get_ias_socket;
pub use crate::ocall::ocall_get_quote;
pub use crate::ocall::ocall_get_update_info;
pub use crate::ocall::ocall_sgx_init_quote;

use std::sync::Arc;
#[derive(Clone)]
pub struct TeeBinder {
    name: String,
    debug_launch: i32,
    enclave_id: sgx_enclave_id_t,
    enclave: Arc<SgxEnclave>,
}

impl TeeBinder {
    pub fn new(name: &str, debug_launch: i32) -> Result<TeeBinder> {
        sgx_launch_check()?;
        let name = name.to_string();
        let enclave = init_enclave(&name, debug_launch)?;
        let enclave_id = enclave.geteid();

        let tee = TeeBinder {
            name,
            debug_launch,
            enclave: Arc::new(enclave),
            enclave_id,
        };

        debug!("EnclaveID: {}", enclave_id);

        let args_info = InitEnclaveInput::default();
        let _ret_info = tee.invoke::<InitEnclaveInput, InitEnclaveOutput>(
            ECallCommand::InitEnclave.into(),
            args_info,
        )?;

        Ok(tee)
    }

    pub fn invoke<U, V>(&self, cmd: u32, args_info: U) -> Result<V>
    where
        U: Serialize,
        V: DeserializeOwned,
    {
        let mut channel = ECallChannel::new(self.enclave_id);
        channel.invoke::<U, V>(cmd, args_info)
    }

    pub fn finalize(&self) -> Result<()> {
        let args_info = FinalizeEnclaveInput::default();
        self.invoke::<FinalizeEnclaveInput, FinalizeEnclaveOutput>(
            ECallCommand::FinalizeEnclave.into(),
            args_info,
        )?;
        //sgx_urts::rsgx_destroy_enclave(enclave_id: sgx_enclave_id_t);
        Ok(())
    }
}

impl Drop for TeeBinder {
    fn drop(&mut self) {
        debug!("Dropping TeeBinder, start finalize().");
        let _ = self.finalize();
    }
}

fn enclave_file_name(enclave_name: &str) -> String {
    format!("{}.{}", enclave_name, ENCLAVE_FILE_SUFFIX)
}

fn enclave_token_name(enclave_name: &str) -> String {
    format!("{}.{}", enclave_name, ENCLAVE_TOKEN_SUFFIX)
}

fn get_token_file(enclave_name: &str) -> Option<PathBuf> {
    env::var("HOME")
        .ok()
        .map(|s| PathBuf::from(s).join(enclave_token_name(enclave_name)))
}

fn try_get_launch_token(
    token_file: Option<PathBuf>,
    launch_token: &mut [u8; TOKEN_LEN],
) -> Result<()> {
    let token_file = token_file.ok_or_else(|| Error::from(ErrorKind::UntrustedAppError))?;
    let mut f = fs::File::open(&token_file).map_err(|e| Error::new(ErrorKind::IoError, e))?;
    match f.read(launch_token) {
        Ok(TOKEN_LEN) => {
            trace!("Read token file successfully.");
            Ok(())
        }
        _ => {
            trace!("Read token file failed.");
            Err(Error::from(ErrorKind::UntrustedAppError))
        }
    }
}

fn try_save_token_to_file(
    token_file: Option<PathBuf>,
    launch_token: &[u8; TOKEN_LEN],
) -> Result<()> {
    let token_file = token_file.ok_or_else(|| Error::from(ErrorKind::UntrustedAppError))?;
    let mut f = fs::File::create(&token_file)?;
    f.write_all(launch_token)?;
    trace!("Save token to {} successfully.", token_file.display());
    Ok(())
}

fn create_misc_attribute() -> sgx_misc_attribute_t {
    sgx_misc_attribute_t {
        secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
        misc_select: 0,
    }
}

fn init_enclave(enclave_name: &str, debug_launch: i32) -> Result<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; TOKEN_LEN];
    let mut launch_token_updated: i32 = 0;

    // Step 1: try to retrieve the launch token saved by last transaction
    //         if there is no token, might create a new one.
    let token_file = get_token_file(enclave_name);

    let _ = try_get_launch_token(token_file.clone(), &mut launch_token);

    // Step 2: call sgx_create_enclave to initialize an enclave instance
    //         change configurations(file/debug/attri) in enclave_config.rs
    let mut misc_attr = create_misc_attribute();
    let enclave_file = enclave_file_name(enclave_name);
    debug!("[+] Enclave File: {}", enclave_file);
    let enclave = SgxEnclave::create(
        enclave_file,
        debug_launch,
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr,
    )?;

    // Step 3: save the launch token if it is updated
    if launch_token_updated != 0 {
        let _ = try_save_token_to_file(token_file.clone(), &launch_token);
    }

    Ok(enclave)
}
