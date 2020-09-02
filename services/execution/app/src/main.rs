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
use std::thread;
use teaclave_service_app_utils::{register_signals, TeaclaveServiceLauncher};

// Use to import ocall
pub use teaclave_file_agent::ocall_handle_file_request;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

fn main() -> Result<()> {
    env_logger::init_from_env(
        env_logger::Env::new()
            .filter_or("TEACLAVE_LOG", "RUST_LOG")
            .write_style_or("TEACLAVE_LOG_STYLE", "RUST_LOG_STYLE"),
    );

    let launcher = Arc::new(TeaclaveServiceLauncher::new(
        PACKAGE_NAME,
        "runtime.config.toml",
    )?);
    let launcher_ref = launcher.clone();
    thread::spawn(move || {
        let _ = launcher_ref.start();
        unsafe { libc::raise(signal_hook::SIGTERM) }
    });

    let term = Arc::new(AtomicBool::new(false));
    register_signals(term.clone()).context("Failed to register signal handler")?;

    while !term.load(Ordering::Relaxed) {
        thread::park();
    }

    launcher.finalize();
    unsafe {
        launcher.destroy(); // force to destroy the enclave
    }

    Ok(())
}
