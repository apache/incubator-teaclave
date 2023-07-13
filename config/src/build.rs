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

#![allow(clippy::all)]
include!(concat!(env!("OUT_DIR"), "/build_config.rs"));

const AUDITOR_PUBLIC_KEYS_LEN: usize = BUILD_CONFIG.auditor_public_keys.len();

/// CA certification of Attestation Service in binary (DER format).
pub const AS_ROOT_CA_CERT: &[u8] = BUILD_CONFIG.as_root_ca_cert;

/// Array of auditor's public keys in binary (DER format), usually used to
/// verify signatures of `enaclave_info.toml`.
pub const AUDITOR_PUBLIC_KEYS: &[&[u8]; AUDITOR_PUBLIC_KEYS_LEN] = BUILD_CONFIG.auditor_public_keys;

/// The valid duration of one attestation report in seconds.
pub const ATTESTATION_VALIDITY_SECS: u64 = BUILD_CONFIG.attestation_validity_secs;

/// gRPC configuration
pub const GRPC_CONFIG: GrpcConfig = BUILD_CONFIG.grpc_config;

macro_rules! def_inbound_services {
    ($name: tt, $service: tt) => {
        /// Array of predefined inbound services, usually used for validate
        /// incoming connections via mutual attestation.
        pub const $name: &[&str; BUILD_CONFIG.inbound.$service.len()] =
            BUILD_CONFIG.inbound.$service;
    };
}

def_inbound_services!(ACCESS_CONTROL_INBOUND_SERVICES, access_control);
def_inbound_services!(AUTHENTICATION_INBOUND_SERVICES, authentication);
def_inbound_services!(MANAGEMENT_INBOUND_SERVICES, management);
def_inbound_services!(SCHEDULER_INBOUND_SERVICES, scheduler);
def_inbound_services!(STORAGE_INBOUND_SERVICES, storage);
