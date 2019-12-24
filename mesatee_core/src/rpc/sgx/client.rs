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

use std::sync::Arc;

use crate::rpc::sgx::EnclaveAttr;

#[cfg(feature = "mesalock_sgx")]
use sgx_types::sgx_sha256_hash_t;
#[cfg(not(feature = "mesalock_sgx"))]
use std::sync::RwLock;
#[cfg(feature = "mesalock_sgx")]
use std::sync::SgxRwLock as RwLock;

use std::collections::HashMap;

use lazy_static::lazy_static;

lazy_static! {
    static ref CLIENT_CONFIG_CACHE: RwLock<ClientConfigCache> =
        { RwLock::new(ClientConfigCache::default()) };
}

#[cfg(feature = "mesalock_sgx")]
#[derive(Default)]
struct ClientConfigCache {
    private_key_sha256: sgx_sha256_hash_t,
    target_configs: HashMap<Arc<EnclaveAttr>, Arc<rustls::ClientConfig>>,
}

#[cfg(not(feature = "mesalock_sgx"))]
#[derive(Default)]
struct ClientConfigCache {
    target_configs: HashMap<Arc<EnclaveAttr>, Arc<rustls::ClientConfig>>,
}

#[cfg(feature = "mesalock_sgx")]
pub(crate) fn get_tls_config(server_attr: Arc<EnclaveAttr>) -> Arc<rustls::ClientConfig> {
    use crate::rpc::sgx::ra::get_ra_cert;

    // To re-use existing TLS cache, we need to first check if the server has
    // updated his RA cert
    let cert_key = get_ra_cert();

    // TODO: add wrapper function
    if let Ok(cfg_cache) = CLIENT_CONFIG_CACHE.try_read() {
        if let Some(cfg) = cfg_cache.target_configs.get(&server_attr) {
            return cfg.clone();
        }
    }

    let certs = vec![rustls::Certificate(cert_key.cert)];
    let privkey = rustls::PrivateKey(cert_key.private_key);

    let mut client_cfg = rustls::ClientConfig::new();
    client_cfg.set_single_client_cert(certs, privkey);
    client_cfg
        .dangerous()
        .set_certificate_verifier(server_attr.clone());
    client_cfg.versions.clear();
    client_cfg.versions.push(rustls::ProtocolVersion::TLSv1_2);

    let final_arc = Arc::new(client_cfg);

    // TODO: add wrapper function
    if let Ok(mut cfg_cache) = CLIENT_CONFIG_CACHE.try_write() {
        if cfg_cache.private_key_sha256 != cert_key.private_key_sha256 {
            *cfg_cache = ClientConfigCache {
                private_key_sha256: cert_key.private_key_sha256,
                target_configs: HashMap::new(),
            }
        }

        let _ = cfg_cache
            .target_configs
            .insert(server_attr, final_arc.clone());
    }

    final_arc
}

#[cfg(not(feature = "mesalock_sgx"))]
pub(crate) fn get_tls_config(server_attr: Arc<EnclaveAttr>) -> Arc<rustls::ClientConfig> {
    // We believe a client from untrusted side do not change his tls cert
    // during single execution.
    if let Ok(cfg_cache) = CLIENT_CONFIG_CACHE.try_read() {
        if let Some(cfg) = cfg_cache.target_configs.get(&server_attr) {
            return cfg.clone();
        }
    }

    let mut client_cfg = rustls::ClientConfig::new();

    client_cfg
        .dangerous()
        .set_certificate_verifier(server_attr.clone());
    client_cfg.versions.clear();
    client_cfg.versions.push(rustls::ProtocolVersion::TLSv1_2);

    let final_arc = Arc::new(client_cfg);

    if let Ok(mut cfg_cache) = CLIENT_CONFIG_CACHE.try_write() {
        let _ = cfg_cache
            .target_configs
            .insert(server_attr, final_arc.clone());
    }

    final_arc
}
