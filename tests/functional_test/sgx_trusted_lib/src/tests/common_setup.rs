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
use acs_client::ACSClient;
use kms_proto::KMSClient;
use mesatee_core::config;
use tdfs_internal_client::TDFSClient;
use tms_internal_client::TMSClient;

pub(crate) fn setup_kms_internal_client() -> KMSClient {
    let target = config::Internal::target_kms();
    KMSClient::new(target).unwrap()
}

pub(crate) fn setup_acs_internal_client() -> ACSClient {
    let target = config::Internal::target_acs();
    ACSClient::new(target).unwrap()
}

pub(crate) fn setup_tdfs_internal_client() -> TDFSClient {
    let target = config::Internal::target_tdfs();
    TDFSClient::new(target).unwrap()
}

pub(crate) fn setup_tms_internal_client() -> TMSClient {
    let target = config::Internal::target_tms();
    TMSClient::new(target).unwrap()
}
