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

#[cfg(feature = "libos")]
pub(crate) mod libos;
pub(crate) mod sgx;
#[cfg(all(feature = "libos", feature = "mesalock_sgx"))]
compile_error!("feature \"mesalock_sgx\" and feature \"libos\" cannot be enabled at the same time");

#[cfg(feature = "libos")]
pub(crate) use libos::{
    occlum::{create_sgx_report_data, get_sgx_dcap_quote, get_sgx_epid_quote},
    PlatformError,
};
#[cfg(feature = "mesalock_sgx")]
pub(crate) use sgx::{create_sgx_isv_enclave_report, get_sgx_quote, init_sgx_quote, PlatformError};

type Result<T> = std::result::Result<T, PlatformError>;

#[cfg(all(feature = "enclave_unit_test", feature = "mesalock_sgx"))]
pub mod tests {
    use super::*;
    pub use sgx::tests::*;
}
