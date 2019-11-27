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
#[cfg(not(feature = "mesalock_sgx"))]
use mesatee_config::MESATEE_SECURITY_CONSTANTS;

#[cfg(not(feature = "mesalock_sgx"))]
use std::sync::RwLock;
#[cfg(feature = "mesalock_sgx")]
use std::sync::SgxRwLock as RwLock;

use std::collections::HashMap;

use lazy_static::lazy_static;

lazy_static! {
    static ref CLIENTCONFIGCACHE: RwLock<HashMap<u64, Arc<rustls::ClientConfig>>> =
        { RwLock::new(HashMap::new()) };
}

#[cfg(not(feature = "mesalock_sgx"))]
pub(crate) fn get_tls_config(server_attr: EnclaveAttr) -> Arc<rustls::ClientConfig> {
    // We believe a client from untrusted side do not change his tls cert
    // during single execution.
    let cattr_hash: u64 = server_attr.calculate_hash();
    if let Ok(cfg_cache) = CLIENTCONFIGCACHE.try_read() {
        if let Some(cfg) = cfg_cache.get(&cattr_hash) {
            return cfg.clone();
        }
    }

    let mut client_cfg = rustls::ClientConfig::new();

    let client_cert = MESATEE_SECURITY_CONSTANTS.client_cert;
    let mut cc_reader = std::io::BufReader::new(&client_cert[..]);

    let client_pkcs8_key = MESATEE_SECURITY_CONSTANTS.client_pkcs8_key;
    let mut client_key_reader = std::io::BufReader::new(&client_pkcs8_key[..]);

    let certs = rustls::internal::pemfile::certs(&mut cc_reader);
    let privk = rustls::internal::pemfile::pkcs8_private_keys(&mut client_key_reader);

    client_cfg.set_single_client_cert(certs.unwrap(), privk.unwrap()[0].clone());

    client_cfg
        .dangerous()
        .set_certificate_verifier(Arc::new(server_attr));
    client_cfg.versions.clear();
    client_cfg.versions.push(rustls::ProtocolVersion::TLSv1_2);

    let final_arc = Arc::new(client_cfg);

    if let Ok(mut cfg_cache) = CLIENTCONFIGCACHE.try_write() {
        let _ = cfg_cache.insert(cattr_hash, final_arc.clone());
    }

    final_arc
}

#[cfg(feature = "mesalock_sgx")]
pub(crate) fn get_tls_config(server_attr: EnclaveAttr) -> Arc<rustls::ClientConfig> {
    use super::calc_hash;
    use crate::rpc::sgx::ra::get_ra_cert;

    // To re-use existing TLS cache, we need to first check if the server has
    // updated his RA cert
    let (cert_key, invalidate_cache) = get_ra_cert();

    // calc_hash generates a u64 for (EnclaveAttr, CertKeyPair)
    // According to the below code, as long as cert_key is unchanged,
    // the `client_cfg` would not change, because it only take two
    // parameters: EnclaveAttr, and CertKeyPair
    let stat_hash: u64 = calc_hash(&server_attr, &cert_key);

    if !invalidate_cache {
        if let Ok(cfg_cache) = CLIENTCONFIGCACHE.try_read() {
            if let Some(cfg) = cfg_cache.get(&stat_hash) {
                return cfg.clone();
            }
        }
    } else {
        match CLIENTCONFIGCACHE.write() {
            Ok(mut cfg_cache) => {
                info!("CLIENTCONFIGCACHE invalidate all config cache!");
                cfg_cache.clear();
            }
            Err(_) => {
                // Poisoned
                // I don't think we should panic here.
                error!("CLIENTCONFIGCACHE invalidate cache failed!");
            }
        }
    }

    let mut certs = std::vec::Vec::new();
    certs.push(rustls::Certificate(cert_key.cert));
    let privkey = rustls::PrivateKey(cert_key.private_key);

    let mut client_cfg = rustls::ClientConfig::new();
    client_cfg.set_single_client_cert(certs, privkey);
    client_cfg
        .dangerous()
        .set_certificate_verifier(Arc::new(server_attr));
    client_cfg.versions.clear();
    client_cfg.versions.push(rustls::ProtocolVersion::TLSv1_2);

    let final_arc = Arc::new(client_cfg);

    if let Ok(mut cfg_cache) = CLIENTCONFIGCACHE.try_write() {
        let _ = cfg_cache.insert(stat_hash, final_arc.clone());
    }

    final_arc
}
