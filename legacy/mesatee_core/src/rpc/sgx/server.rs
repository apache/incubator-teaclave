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

use crate::Error;
use crate::ErrorKind;
use crate::Result;
use teaclave_attestation::verifier::SgxQuoteVerifier;

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
    target_configs: HashMap<Arc<SgxQuoteVerifier>, Arc<rustls::ServerConfig>>,
}

pub fn get_tls_config(
    client_verifier: &Option<SgxQuoteVerifier>,
) -> Result<Arc<rustls::ServerConfig>> {
    use crate::rpc::sgx::ra::get_current_ra_credential;

    let ra_credential = get_current_ra_credential();

    let client_verifier = match client_verifier {
        Some(attr) => Arc::new(attr.clone()),
        None => {
            let certs = vec![rustls::Certificate(ra_credential.cert)];
            let privkey = rustls::PrivateKey(ra_credential.private_key);
            // Build a default authenticator which allow every authenticated client
            let authenticator = rustls::NoClientAuth::new();
            let mut cfg = rustls::ServerConfig::new(authenticator);
            cfg.set_single_cert(certs, privkey)
                .map_err(|_| Error::from(ErrorKind::TLSError))?;
            return Ok(Arc::new(cfg));
        }
    };

    if let Ok(cfg_cache) = SERVER_CONFIG_CACHE.try_read() {
        if let Some(cfg) = cfg_cache.target_configs.get(&client_verifier) {
            // Hit Cache. Be quick!
            return Ok(cfg.clone());
        }
    }

    let certs = vec![rustls::Certificate(ra_credential.cert)];
    let privkey = rustls::PrivateKey(ra_credential.private_key);

    let mut server_cfg = rustls::ServerConfig::new(client_verifier.clone());
    server_cfg
        .set_single_cert(certs, privkey)
        .map_err(|_| Error::from(ErrorKind::TLSError))?;

    let final_arc = Arc::new(server_cfg);

    if let Ok(mut cfg_cache) = SERVER_CONFIG_CACHE.try_write() {
        if cfg_cache.private_key_sha256 != ra_credential.private_key_sha256 {
            *cfg_cache = ServerConfigCache {
                private_key_sha256: ra_credential.private_key_sha256,
                target_configs: HashMap::new(),
            }
        }
        let _ = cfg_cache
            .target_configs
            .insert(client_verifier, final_arc.clone()); // Overwrite
    }

    Ok(final_arc)
}
