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
use std::path::Path;

// check prerequisites to make the launching process smoother
// the launching may still fail even after passing the check
pub fn sgx_launch_check() -> Result<()> {
    // check the existence of ias_spid.txt and ias_key.txt
    if !cfg!(sgx_sim)
        && (!Path::new(&MESATEE_CONFIG.ias_client_spid_path).is_file()
            || !Path::new(&MESATEE_CONFIG.ias_client_key_path).is_file())
    {
        error!("SGX launch check failed: {} or {} does NOT exist. Please follow \"How to Run (SGX)\" in README to obtain.",
        "ias_spid.txt", "ias_key.txt");
        return Err(ErrorKind::IASClientKeyCertError.into());
    }
    Ok(())
}
