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

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use teaclave_binder::proto::{ECallCommand, StartServiceInput, StartServiceOutput};
use teaclave_binder::TeeBinder;
use teaclave_config::RuntimeConfig;
use teaclave_types::TeeServiceResult;

pub struct TeaclaveServiceLauncher {
    tee: TeeBinder,
    config: RuntimeConfig,
}

impl TeaclaveServiceLauncher {
    pub fn new<P: AsRef<Path>>(package_name: &str, config_path: P) -> Result<Self> {
        let config = RuntimeConfig::from_toml(config_path.as_ref())
            .context("Failed to load config file.")?;
        let tee = TeeBinder::new(package_name).context("Failed to new the enclave.")?;
        Ok(Self { tee, config })
    }

    pub fn start(&self) -> Result<String> {
        let input = StartServiceInput::new(self.config.clone());
        let command = ECallCommand::StartService;
        match self
            .tee
            .invoke::<StartServiceInput, TeeServiceResult<StartServiceOutput>>(command, input)
        {
            Err(e) => bail!("TEE invocation error: {:?}", e),
            Ok(Err(e)) => bail!("Service exit with error: {:?}", e),
            _ => Ok(String::from("Service successfully exit")),
        }
    }

    pub fn finalize(&self) {
        self.tee.finalize();
    }

    pub unsafe fn destroy(&self) {
        self.tee.destroy();
    }
}

pub fn register_signals(term: Arc<AtomicBool>) -> Result<()> {
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
