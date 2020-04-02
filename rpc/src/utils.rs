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

#[allow(dead_code)]
pub(crate) fn get_tcs_num() -> usize {
    if sgx_trts::enclave::rsgx_is_supported_EDMM() {
        sgx_trts::enclave::SgxGlobalData::new().get_dyn_tcs_num() as usize
    } else {
        (sgx_trts::enclave::SgxGlobalData::new().get_tcs_max_num() - 1) as usize
    }
}
