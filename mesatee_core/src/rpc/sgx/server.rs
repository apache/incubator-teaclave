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

use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;

use crate::rpc::sgx::EnclaveAttr;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

use sgx_types::sgx_sha256_hash_t;
use std::sync::SgxRwLock as RwLock;

use lazy_static::lazy_static;

lazy_static! {
    static ref SERVER_CONFIG_CACHE: RwLock<ServerConfigCache> =
        { RwLock::new(ServerConfigCache::default()) };
}

#[derive(Default)]
struct ServerConfigCache {
    private_key_sha256: sgx_sha256_hash_t,
    target_configs: HashMap<Arc<EnclaveAttr>, Arc<rustls::ServerConfig>>,
}

pub(crate) fn get_tls_config(
    client_attr: Option<EnclaveAttr>,
) -> Result<Arc<rustls::ServerConfig>> {
    use crate::rpc::sgx::ra::get_ra_cert;

    // To re-use existing TLS cache, we need to first check if the server has
    // updated his RA cert
    let cert_key = get_ra_cert();

    let client_attr = match client_attr {
        Some(attr) => Arc::new(attr),
        None => {
            let mut certs = Vec::new();
            certs.push(rustls::Certificate(cert_key.cert));
            let privkey = rustls::PrivateKey(cert_key.private_key);
            // Build a default authenticator which allow every authenticated client
            let authenticator = rustls::NoClientAuth::new();
            let mut cfg = rustls::ServerConfig::new(authenticator);
            cfg.set_single_cert(certs, privkey)
                .map_err(|_| Error::from(ErrorKind::TLSError))?;
            return Ok(Arc::new(cfg));
        }
    };

    if let Ok(cfg_cache) = SERVER_CONFIG_CACHE.try_read() {
        if let Some(cfg) = cfg_cache.target_configs.get(&client_attr) {
            // Hit Cache. Be quick!
            return Ok(cfg.clone());
        }
    }

    let certs = vec![rustls::Certificate(cert_key.cert)];
    let privkey = rustls::PrivateKey(cert_key.private_key);

    let mut server_cfg = rustls::ServerConfig::new(client_attr.clone());
    server_cfg
        .set_single_cert(certs, privkey)
        .map_err(|_| Error::from(ErrorKind::TLSError))?;

    let final_arc = Arc::new(server_cfg);

    if let Ok(mut cfg_cache) = SERVER_CONFIG_CACHE.try_write() {
        if cfg_cache.private_key_sha256 != cert_key.private_key_sha256 {
            *cfg_cache = ServerConfigCache {
                private_key_sha256: cert_key.private_key_sha256,
                target_configs: HashMap::new(),
            }
        }
        let _ = cfg_cache
            .target_configs
            .insert(client_attr, final_arc.clone()); // Overwrite
    }

    Ok(final_arc)
}
