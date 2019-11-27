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

//! rpc support for MesaTEE-SGX

// Insert std prelude in the top for the sgx feature
use serde::{de::DeserializeOwned, Serialize};
#[cfg(feature = "mesalock_sgx")]
use sgx_types::c_int;
use sgx_types::SGX_HASH_SIZE;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{self, Read, Write};
use std::marker::PhantomData;
use std::net::TcpStream;
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

#[cfg(feature = "mesalock_sgx")]
use crate::rpc::{EnclaveService, RpcServer};

use crate::rpc::RpcClient;
use crate::Result;

use mesatee_config::MESATEE_CONFIG;
use mesatee_config::MESATEE_SECURITY_CONSTANTS;

#[macro_use]
mod fail;

mod utils;
pub use utils::load_and_verify_enclave_info;

pub mod client;
#[cfg(feature = "mesalock_sgx")]
pub mod server;

#[macro_use]
mod cert;
pub mod auth;
pub use auth::*;
#[cfg(feature = "mesalock_sgx")]
mod ra;

// Export this function for sgx enclave initialization
#[cfg(feature = "mesalock_sgx")]
pub fn prelude() {
    ra::get_ra_cert();
}

pub type SgxMeasure = [u8; SGX_HASH_SIZE];

pub(crate) fn load_presigned_enclave_info() -> HashMap<String, (SgxMeasure, SgxMeasure)> {
    let enclave_info_path = MESATEE_CONFIG.audited_enclave_info_path.as_path();
    let enclave_signers: Vec<(&'static [u8], &std::path::Path)> = vec![
        (
            MESATEE_SECURITY_CONSTANTS.audited_enclave_pubkey_a,
            &MESATEE_CONFIG.auditor_a_signature_path,
        ),
        (
            MESATEE_SECURITY_CONSTANTS.audited_enclave_pubkey_b,
            &MESATEE_CONFIG.auditor_b_signature_path,
        ),
        (
            MESATEE_SECURITY_CONSTANTS.audited_enclave_pubkey_c,
            &MESATEE_CONFIG.auditor_c_signature_path,
        ),
    ];

    utils::load_and_verify_enclave_info(enclave_info_path, enclave_signers.as_slice())
}

#[derive(Clone)]
pub struct EnclaveAttr {
    pub measures: Vec<(SgxMeasure, SgxMeasure)>,
    pub quote_checker: fn(&SgxQuote) -> bool,
}

impl PartialEq for EnclaveAttr {
    fn eq(&self, other: &EnclaveAttr) -> bool {
        self.quote_checker as usize == other.quote_checker as usize
            && self.measures == other.measures
    }
}

impl Eq for EnclaveAttr {}

impl Hash for EnclaveAttr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (m1, m2) in &self.measures {
            m1.hash(state);
            m2.hash(state);
        }
        (self.quote_checker as usize).hash(state);
    }
}

#[cfg(feature = "mesalock_sgx")]
pub(crate) fn calc_hash(ea: &EnclaveAttr, crp: &ra::CertKeyPair) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut s = DefaultHasher::new();
    ea.hash(&mut s);
    crp.hash(&mut s);
    s.finish()
}

impl EnclaveAttr {
    #[cfg(not(feature = "mesalock_sgx"))]
    fn calculate_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

    fn check_in_cert_quote(&self, cert_der: &[u8]) -> bool {
        if cfg!(sgx_sim) {
            return true;
        }

        let quote_result = auth::extract_sgx_quote_from_mra_cert(&cert_der);
        let quote: SgxQuote = match quote_result {
            Err(_) => {
                return false;
            }
            Ok(quote) => quote,
        };

        let this_mr_signer = &quote.body.report_body.mr_signer;
        let this_mr_enclave = &quote.body.report_body.mr_enclave;

        let checksum_match = self.measures.iter().any(|(mr_signer, mr_enclave)| {
            mr_signer == this_mr_signer && mr_enclave == this_mr_enclave
        });

        if !checksum_match {
            info!("sgx enclave measure mismatch. try matching with functional_test");

            use crate::config::ENCLAVE_IDENTITIES;

            // TODO: For testing, an enclave called functional_test will communicate
            // with each MesaTEE component. We have to explicitly add its measures into
            // the whitelist. Ideally, this piece of code should not materialize in
            // release build. Find a way to make this configurable.
            let (test_mr_signer, test_mr_enclave) =
                ENCLAVE_IDENTITIES.get("functional_test").unwrap();
            if this_mr_signer != test_mr_signer || this_mr_enclave != test_mr_enclave {
                error!("sgx enclave measure mismatch");
                return false;
            }
        }

        (self.quote_checker)(&quote)
    }
}

impl rustls::ServerCertVerifier for EnclaveAttr {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        certs: &[rustls::Certificate],
        _hostname: webpki::DNSNameRef,
        _ocsp: &[u8],
    ) -> std::result::Result<rustls::ServerCertVerified, rustls::TLSError> {
        // This call automatically verifies certificate signature
        if certs.len() != 1 {
            return Err(rustls::TLSError::NoCertificatesPresented);
        }
        if self.check_in_cert_quote(&certs[0].0) {
            Ok(rustls::ServerCertVerified::assertion())
        } else {
            Err(rustls::TLSError::WebPKIError(
                webpki::Error::ExtensionValueInvalid,
            ))
        }
    }
}

impl rustls::ClientCertVerifier for EnclaveAttr {
    fn client_auth_root_subjects(&self) -> rustls::DistinguishedNames {
        rustls::DistinguishedNames::new()
    }

    fn verify_client_cert(
        &self,
        certs: &[rustls::Certificate],
    ) -> std::result::Result<rustls::ClientCertVerified, rustls::TLSError> {
        // This call automatically verifies certificate signature
        if certs.len() != 1 {
            return Err(rustls::TLSError::NoCertificatesPresented);
        }
        if self.check_in_cert_quote(&certs[0].0) {
            Ok(rustls::ClientCertVerified::assertion())
        } else {
            Err(rustls::TLSError::WebPKIError(
                webpki::Error::ExtensionValueInvalid,
            ))
        }
    }
}

#[cfg(feature = "mesalock_sgx")]
pub struct PipeConfig {
    pub fd: c_int,
    pub retry: u32,
    // the SGX server can optionally verify the identity of the client
    pub client_attr: Option<EnclaveAttr>,
}

#[cfg(feature = "mesalock_sgx")]
pub struct Pipe<U, V, X> {
    inner: rustls::StreamOwned<rustls::ServerSession, TcpStream>,
    u: PhantomData<U>,
    v: PhantomData<V>,
    x: PhantomData<X>,
}

#[cfg(feature = "mesalock_sgx")]
impl<U, V, X> Read for Pipe<U, V, X> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

#[cfg(feature = "mesalock_sgx")]
impl<U, V, X> Write for Pipe<U, V, X> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(feature = "mesalock_sgx")]
impl<U, V, X> RpcServer<U, V, X> for Pipe<U, V, X>
where
    U: DeserializeOwned,
    V: Serialize,
    X: EnclaveService<U, V>,
{
    type Config = PipeConfig;
    fn start(config: Self::Config) -> Result<Self> {
        let tcp = TcpStream::new(config.fd)?;

        // TCP set nodelay should not affect the success of this function
        // We do not care if it is successful or not.
        // Just do it.
        let _ = tcp.set_nodelay(true);

        // TODO: Due to switching to the SDK-style design, performing an
        // initial RA at enclave start is not longer a viable design. Need
        // to refactor the related API.
        let rustls_server_cfg = server::get_tls_config(config.client_attr)?;
        let sess = rustls::ServerSession::new(&rustls_server_cfg);

        Ok(Pipe {
            inner: rustls::StreamOwned::new(sess, tcp),
            u: PhantomData::<U>,
            v: PhantomData::<V>,
            x: PhantomData::<X>,
        })
    }

    // Use default implementation
    // fn serve(&mut self, mut s: X) -> Result<()>;
}

pub struct PipeClient<U, V> {
    inner: rustls::StreamOwned<rustls::ClientSession, TcpStream>,
    u: PhantomData<U>,
    v: PhantomData<V>,
}

pub struct PipeClientConfig {
    pub tcp: TcpStream,
    pub hostname: webpki::DNSName,
    pub server_attr: EnclaveAttr,
}

impl<U, V> Read for PipeClient<U, V> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<U, V> Write for PipeClient<U, V> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<U, V> RpcClient<U, V> for PipeClient<U, V>
where
    U: Serialize,
    V: DeserializeOwned,
{
    type Config = PipeClientConfig;
    fn open(config: Self::Config) -> Result<Self> {
        let rustls_client_cfg = client::get_tls_config(config.server_attr);
        let sess = rustls::ClientSession::new(&rustls_client_cfg, config.hostname.as_ref());

        Ok(PipeClient {
            inner: rustls::StreamOwned::new(sess, config.tcp),
            u: PhantomData::<U>,
            v: PhantomData::<V>,
        })
    }

    // use default implementation
    // fn invoke(&mut self, input: U) -> Result<V>;
}
