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
use std::hash::{Hash, Hasher};
use std::io::{self, Read, Write};
use std::marker::PhantomData;
use std::net::TcpStream;
use std::sync::Arc;

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

#[cfg(feature = "mesalock_sgx")]
use crate::rpc::{EnclaveService, RpcServer};

use crate::rpc::RpcClient;
use crate::Result;

use teaclave_config::build_config::BUILD_CONFIG;
use teaclave_attestation;
use teaclave_attestation::quote::SgxQuote;
use teaclave_utils;
use teaclave_utils::EnclaveMeasurement;

pub mod client;
#[cfg(feature = "mesalock_sgx")]
pub mod server;

#[cfg(feature = "mesalock_sgx")]
mod ra;

// Export this function for sgx enclave initialization
#[cfg(feature = "mesalock_sgx")]
pub fn prelude() -> Result<()> {
    // Hard coded RACredential validity in seconds for all enclave.
    // We may allow each enclave to setup its own validity in the future.
    ra::init_ra_credential(86400u64)
}

#[derive(Clone)]
pub struct EnclaveAttr {
    pub measures: Vec<EnclaveMeasurement>,
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
        for m in &self.measures {
            m.mr_enclave.hash(state);
            m.mr_signer.hash(state);
        }
        (self.quote_checker as usize).hash(state);
    }
}

impl EnclaveAttr {
    fn check_in_cert_quote(&self, cert_der: &[u8]) -> bool {
        if cfg!(sgx_sim) {
            return true;
        }

        let quote_result = teaclave_attestation::quote::extract_sgx_quote_from_mra_cert(
            &cert_der,
            BUILD_CONFIG.ias_root_ca_cert,
        );
        let quote: teaclave_attestation::quote::SgxQuote = match quote_result {
            Err(_) => {
                return false;
            }
            Ok(quote) => quote,
        };

        // Enclave measures are not tested in test mode since we have
        // a dedicated test enclave not known to production enclaves
        if cfg!(test_mode) {
            return (self.quote_checker)(&quote);
        }

        let this_mr_signer = &quote.body.report_body.mr_signer;
        let this_mr_enclave = &quote.body.report_body.mr_enclave;

        let checksum_match = self
            .measures
            .iter()
            .any(|m| &m.mr_signer == this_mr_signer && &m.mr_enclave == this_mr_enclave);

        checksum_match && (self.quote_checker)(&quote)
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
    U: DeserializeOwned + std::fmt::Debug,
    V: Serialize + std::fmt::Debug,
    X: EnclaveService<U, V>,
{
    type Config = PipeConfig;
    fn start(config: &Self::Config) -> Result<Self> {
        let tcp = TcpStream::new(config.fd)?;

        // TCP set nodelay should not affect the success of this function
        // We do not care if it is successful or not.
        // Just do it.
        let _ = tcp.set_nodelay(true);

        // TODO: Due to switching to the SDK-style design, performing an
        // initial RA at enclave start is not longer a viable design. Need
        // to refactor the related API.
        let rustls_server_cfg = server::get_tls_config(&config.client_attr)?;
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
        let rustls_client_cfg = client::get_tls_config(Arc::new(config.server_attr));
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
