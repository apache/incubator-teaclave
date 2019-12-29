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

#![allow(clippy::unreadable_literal, clippy::redundant_closure)]

// This entire file is solely used for the sgx environment
use std::prelude::v1::*;

use sgx_tcrypto::rsgx_sha256_slice;
use sgx_types::*;

use std::sync::SgxRwLock;
use std::time::{self, SystemTime};
use std::untrusted::time::SystemTimeEx;

use lazy_static::lazy_static;

use crate::{Error, ErrorKind, Result};

use crate::config::runtime_config;
use teaclave_ra;

lazy_static! {
    static ref RACACHE: SgxRwLock<RACache> = {
        SgxRwLock::new(RACache {
            ra_credential: RACredential::default(),
            gen_time: SystemTime::UNIX_EPOCH,
            validity: time::Duration::from_secs(0),
        })
    };
}

/// Certificate and public key in DER format
#[derive(Clone, Hash, Default)]
pub(crate) struct RACredential {
    pub cert: Vec<u8>,
    pub private_key: Vec<u8>,
    pub private_key_sha256: sgx_sha256_hash_t,
}

#[derive(Clone)]
struct RACache {
    ra_credential: RACredential,
    gen_time: SystemTime,
    validity: time::Duration,
}

pub(crate) fn init_ra_credential(valid_secs: u64) -> Result<()> {
    match RACache::new(valid_secs) {
        Ok(new_entry) => {
            *RACACHE.write().unwrap() = new_entry;
            Ok(())
        }
        Err(e) => {
            error!("Cannot initialize RACredential: {:?}", e);
            Err(Error::from(ErrorKind::RAInternalError))
        }
    }
}

pub(crate) fn get_current_ra_credential() -> RACredential {
    // Check if the global cert valid
    // If valid, use it directly
    // If invalid, update it before use.
    // Generate Keypair

    // 1. Check if the global cert valid
    //    Need block read here. It should wait for writers to complete
    {
        // Unwrapping failing means the RwLock is poisoned.
        // Simple crash in that case.
        let g_cache = RACACHE.read().unwrap();
        if g_cache.is_valid() {
            return g_cache.ra_credential.clone();
        }
    }

    // 2. Do the update

    // Unwrapping failing means the RwLock is poisoned.
    // Simple crash in that case.
    let mut g_cache = RACACHE.write().unwrap();

    // Here is the 100% serialized access to SVRCONFIG
    // No other reader/writer exists in this branch
    // Toc tou check
    if g_cache.is_valid() {
        return g_cache.ra_credential.clone();
    }

    // Do the renew
    match RACache::new(g_cache.validity.as_secs()) {
        // If RA renewal fails, we do not crash for the following reasons.
        // 1. Crashing the enclave causes most data to be lost permanently,
        //    since we do not have persistent key-value storage yet. On the
        //    other hand, RA renewal failure may be temporary. We still have
        //    a chance to recover from this failure in the future.
        // 2. If renewal failed, the old certificate is used, the the client
        //    can decide if they want to keep talking to the enclave.
        // 3. The certificate has a 90 days valid duration. If RA keeps
        //    failing for 90 days, the enclave itself will not serve any
        //    client.
        Err(e) => {
            error!(
                "RACredential renewal failed, use existing credential: {:?}",
                e
            );
        }
        Ok(new_cache) => *g_cache = new_cache,
    };

    g_cache.ra_credential.clone()
}

impl RACredential {
    fn generate_and_endorse() -> Result<RACredential> {
        let key_pair = teaclave_ra::key::Secp256k1KeyPair::new().map_err(|_| Error::from(ErrorKind::RAInternalError))?;
        let report = match teaclave_ra::SgxRaReport::new(key_pair.pub_k, &runtime_config().env.ias_key, &runtime_config().env.ias_spid, false) {
            Ok(r) => r,
            Err(e) => {
                error!("{:?}", e);
                return Err(Error::from(ErrorKind::RAInternalError));
            }
        };
        let payload = [report.report, report.signature, report.signing_cert].join("|");
        let cert_der =
            key_pair.create_cert_with_extension("Teaclave", "Teaclave", &payload.as_bytes());
        let prv_key_der = key_pair.private_key_into_der();
        let sha256 = rsgx_sha256_slice(&prv_key_der)?;

        Ok(RACredential {
            cert: cert_der,
            private_key: prv_key_der,
            private_key_sha256: sha256,
        })
    }
}

impl RACache {
    fn new(valid_secs: u64) -> Result<RACache> {
        let ra_credential = RACredential::generate_and_endorse()?;
        let gen_time = SystemTime::now();
        let validity = time::Duration::from_secs(valid_secs);
        Ok(RACache {
            ra_credential,
            gen_time,
            validity,
        })
    }

    fn is_valid(&self) -> bool {
        let dur = SystemTime::now().duration_since(self.gen_time);
        dur.is_ok() && dur.unwrap() < self.validity
    }
}
