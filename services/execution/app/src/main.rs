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

use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use teaclave_binder::proto::{ECallCommand, StartServiceInput, StartServiceOutput};
use teaclave_binder::TeeBinder;
use teaclave_config::RuntimeConfig;
use teaclave_types::TeeServiceResult;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

pub use teaclave_file_agent::ocall_handle_file_request;

fn register_signals(term: Arc<AtomicBool>) -> Result<()> {
    for signal in &[
        signal_hook::SIGTERM,
        signal_hook::SIGINT,
        signal_hook::SIGHUP,
    ] {
        let term_ref = term.clone();
        let thread = std::thread::current();
        unsafe {
            signal_hook::register(*signal, move || {
                term_ref.store(true, Ordering::Relaxed);
                thread.unpark();
            })?;
        }
    }

    Ok(())
}

fn start_enclave_service(tee: Arc<TeeBinder>, config: RuntimeConfig) {
    let input = StartServiceInput::new(config);
    let command = ECallCommand::StartService;
    match tee.invoke::<StartServiceInput, TeeServiceResult<StartServiceOutput>>(command, input) {
        Err(e) => {
            eprintln!("TEE invocation error: {:?}", e);
        }
        Ok(Err(e)) => {
            eprintln!("Service exit with error: {:?}", e);
        }
        _ => {
            println!("Service successfully exit");
        }
    }

    unsafe { libc::raise(signal_hook::SIGTERM) };
}

fn main() -> Result<()> {
    env_logger::init();

    let tee = Arc::new(TeeBinder::new(PACKAGE_NAME).context("Failed to new the enclave.")?);
    let config = teaclave_config::RuntimeConfig::from_toml("runtime.config.toml")
        .context("Failed to load config file.")?;

    let tee_ref = tee.clone();
    std::thread::spawn(move || {
        start_enclave_service(tee_ref, config);
    });

    let term = Arc::new(AtomicBool::new(false));
    register_signals(term.clone()).context("Failed to register signal handler")?;

    while !term.load(Ordering::Relaxed) {
        std::thread::park();
    }

    tee.finalize();

    Ok(())
}
