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

use crate::error::{ErrorKind, Result};
use mesatee_config::MESATEE_CONFIG;

// check prerequisites to make the launching process smoother
// the launching may still fail even after passing the check
pub fn sgx_launch_check() -> Result<()> {
    // check the existence of env var specified in config.toml
    if !cfg!(sgx_sim)
        && (std::env::var(&MESATEE_CONFIG.ias_client_spid_envvar).is_ok()
            || std::env::var(&MESATEE_CONFIG.ias_client_key_envvar).is_ok())
    {
        error!("SGX launch check failed: Env var for IAS SPID or IAS KEY does NOT exist. Please follow \"How to Run (SGX)\" in README to obtain, and specify the value in environment variables and put the names of environment variables in config.toml.");
        return Err(ErrorKind::IASClientKeyCertError.into());
    }
    Ok(())
}
