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

use std::env;
use teaclave_config;

pub fn run_tests() -> bool {
    teaclave_test_utils::run_tests!(test_runtime_config, test_runtime_config_with_env_vars)
}

fn test_runtime_config() {
    env::remove_var("AS_KEY");
    env::remove_var("AS_SPID");
    let config =
        teaclave_config::RuntimeConfig::from_toml("./fixtures/runtime.config.toml").unwrap();
    let authentication_config = config.api_endpoints.authentication;
    assert_eq!(
        authentication_config.listen_address,
        "0.0.0.0:7776".parse().unwrap()
    );
    let storage_config = config.internal_endpoints.storage;
    assert_eq!(
        storage_config.listen_address,
        "0.0.0.0:17778".parse().unwrap()
    );
    assert_eq!(
        storage_config.inbound_services,
        Some(vec!["frontend".to_string()])
    );

    assert_eq!(config.attestation.key, "ias_key_AAAABBBBCCCCDDDDEEEEFFFF");
    assert_eq!(config.attestation.spid, "ias_spid_AAAABBBBCCCCDDDDEEEEFFF");

    assert_eq!(
        config.audit.auditor_signatures_bytes.as_ref().unwrap()[0],
        b"godzilla.sign.sha256"
    )
}

fn test_runtime_config_with_env_vars() {
    env::set_var("AS_URL", "xxx.yy.zz:8080");
    env::set_var("AS_ALGO", "sgx_epid");
    env::set_var("AS_KEY", "12345678901234567890123456789012");
    env::set_var("AS_SPID", "90123456789012345678901234567890");
    let config =
        teaclave_config::RuntimeConfig::from_toml("./fixtures/runtime.config.toml").unwrap();
    assert_eq!(config.attestation.url, "xxx.yy.zz:8080");
    assert_eq!(config.attestation.key, "12345678901234567890123456789012");
    assert_eq!(config.attestation.spid, "90123456789012345678901234567890");
}
