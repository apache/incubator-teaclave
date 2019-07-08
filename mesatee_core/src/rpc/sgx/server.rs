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

use std::collections::HashMap;
use std::sync::Arc;
#[cfg(not(feature = "mesalock_sgx"))]
use std::sync::RwLock;
#[cfg(feature = "mesalock_sgx")]
use std::sync::SgxRwLock as RwLock;
use std::vec::Vec;

use mesatee_config::MESATEE_SECURITY_CONSTANTS;

use crate::rpc::sgx::EnclaveAttr;
use crate::Error;
use crate::ErrorKind;
use crate::Result;

use lazy_static::lazy_static;

lazy_static! {
    static ref SVRCONFIGCACHE: RwLock<HashMap<u64, Arc<rustls::ServerConfig>>> =
        { RwLock::new(HashMap::new()) };
}

pub(crate) fn get_tls_config(
    client_attr: Option<EnclaveAttr>,
) -> Result<Arc<rustls::ServerConfig>> {
    use super::calc_hash;
    use crate::rpc::sgx::ra::get_ra_cert;

    // To re-use existing TLS cache, we need to first check if the server has
    // updated his RA cert
    let (cert_key, invalidate_cache) = get_ra_cert();

    let attr_with_hash = client_attr.map(|attr| {
        // This branch is for accepting socket from Enclaves
        // calc_hash generates a u64 for (EnclaveAttr, CertKeyPair)
        // According to the below code, as long as cert_key is unchanged,
        // the `mutual_cfg` would not change, because it only take two
        // parameters: EnclaveAttr, and CertKeyPair
        let hash = calc_hash(&attr, &cert_key);
        (attr, hash)
    });

    // invalidate_cache is true iff. ra is renewed in the above func call
    // if ra cert is pulled from cache, then we can try to do it quickly.
    if !invalidate_cache {
        if let Some(&(_, new_hash)) = attr_with_hash.as_ref() {
            if let Ok(cfg_cache) = SVRCONFIGCACHE.try_read() {
                if let Some(cfg) = cfg_cache.get(&new_hash) {
                    // Everything matched. Be quick!
                    return Ok(cfg.clone());
                }
            }
        };
    } else {
        // ra cert is updated. so we need to invalidate the cache
        // THIS IS BLOCKING!
        match SVRCONFIGCACHE.write() {
            Ok(mut cfg_cache) => {
                info!("SVRCONFIGCACHE invalidate all config cache!");
                cfg_cache.clear();
            }
            Err(x) => {
                // Poisoned
                error!("SVRCONFIGCACHE invalidate cache failed {}!", x);
            }
        }
    }

    let root_ca_bin = MESATEE_SECURITY_CONSTANTS.root_ca_bin;
    let mut ca_reader = std::io::BufReader::new(&root_ca_bin[..]);
    let mut rc_store = rustls::RootCertStore::empty();

    // Build a root ca storage
    rc_store
        .add_pem_file(&mut ca_reader)
        .map_err(|_| Error::from(ErrorKind::TLSError))?;

    let mut certs = Vec::new();
    certs.push(rustls::Certificate(cert_key.cert));
    let privkey = rustls::PrivateKey(cert_key.private_key);

    if let Some((client_attr, stat_hash)) = attr_with_hash {
        // We assigned Some(u64) to new_hash when client_attr is some.
        // So in this branch, new_hash should always be Some(u64)
        let mut mutual_cfg = rustls::ServerConfig::new(Arc::new(client_attr));
        mutual_cfg
            .set_single_cert(certs, privkey)
            .map_err(|_| Error::from(ErrorKind::TLSError))?;

        let final_arc = Arc::new(mutual_cfg); // Create an Arc

        if let Ok(mut cfg_cache) = SVRCONFIGCACHE.try_write() {
            let _ = cfg_cache.insert(stat_hash, final_arc.clone()); // Overwrite
        }

        Ok(final_arc)
    } else {
        // Build a default authenticator which allow every authenticated client
        let authenticator = rustls::AllowAnyAuthenticatedClient::new(rc_store);
        let mut cfg = rustls::ServerConfig::new(authenticator);
        cfg.set_single_cert(certs, privkey)
            .map_err(|_| Error::from(ErrorKind::TLSError))?;

        Ok(Arc::new(cfg))
    }
}
