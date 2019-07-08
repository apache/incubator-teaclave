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
use kms_client::KMSClient;
use mesatee_core::config;
use tdfs_internal_client::TDFSClient;
use tms_internal_client::TMSClient;

pub(crate) fn setup_kms_internal_client() -> KMSClient {
    let target = config::Internal::target_kms();
    KMSClient::new(target).unwrap()
}

pub(crate) fn setup_tdfs_internal_client() -> TDFSClient {
    let target = config::Internal::target_tdfs();
    TDFSClient::new(target).unwrap()
}

pub(crate) fn setup_tms_internal_client() -> TMSClient {
    let target = config::Internal::target_tms();
    TMSClient::new(target).unwrap()
}
