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

use crate::quote::SgxQuote;
use std::hash::{Hash, Hasher};
use std::vec::Vec;
use teaclave_config::build_config::BUILD_CONFIG;
use teaclave_types::EnclaveMeasurement;

#[derive(Clone)]
pub struct EnclaveAttr {
    pub measures: Vec<EnclaveMeasurement>,
}

impl PartialEq for EnclaveAttr {
    fn eq(&self, other: &EnclaveAttr) -> bool {
        self.measures == other.measures
    }
}

impl Hash for EnclaveAttr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for m in &self.measures {
            m.mr_enclave.hash(state);
            m.mr_signer.hash(state);
        }
    }
}

#[derive(Clone)]
pub struct SgxQuoteVerifier {
    pub enclave_attr: EnclaveAttr,
    pub verifier: fn(&SgxQuote) -> bool,
}

impl PartialEq for SgxQuoteVerifier {
    fn eq(&self, other: &SgxQuoteVerifier) -> bool {
        self.verifier as usize == other.verifier as usize && self.enclave_attr == other.enclave_attr
    }
}

impl Eq for SgxQuoteVerifier {}

impl Hash for SgxQuoteVerifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.enclave_attr.hash(state);
        (self.verifier as usize).hash(state);
    }
}

fn universal_quote_verifier(quote: &SgxQuote) -> bool {
    quote.status != crate::quote::SgxQuoteStatus::UnknownBadStatus
}

impl SgxQuoteVerifier {
    pub fn new(enclave_attr: EnclaveAttr) -> Self {
        Self {
            enclave_attr,
            verifier: universal_quote_verifier,
        }
    }

    fn verify_measures(&self, quote: &SgxQuote) -> bool {
        let this_mr_signer = quote.body.report_body.mr_signer;
        let this_mr_enclave = quote.body.report_body.mr_enclave;

        self.enclave_attr
            .measures
            .iter()
            .any(|m| m.mr_signer == this_mr_signer && m.mr_enclave == this_mr_enclave)
    }

    fn verify_cert(&self, cert_der: &[u8]) -> bool {
        if cfg!(sgx_sim) {
            return true;
        }

        let quote = match SgxQuote::extract_verified_quote(&cert_der, BUILD_CONFIG.ias_root_ca_cert)
        {
            Ok(quote) => quote,
            Err(_) => {
                return false;
            }
        };

        // Enclave measures are not tested in test mode since we have
        // a dedicated test enclave not known to production enclaves
        if cfg!(test_mode) {
            return (self.verifier)(&quote);
        }

        self.verify_measures(&quote) && (self.verifier)(&quote)
    }
}

impl rustls::ServerCertVerifier for SgxQuoteVerifier {
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
        if self.verify_cert(&certs[0].0) {
            Ok(rustls::ServerCertVerified::assertion())
        } else {
            Err(rustls::TLSError::WebPKIError(
                webpki::Error::ExtensionValueInvalid,
            ))
        }
    }
}

impl rustls::ClientCertVerifier for SgxQuoteVerifier {
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
        if self.verify_cert(&certs[0].0) {
            Ok(rustls::ClientCertVerified::assertion())
        } else {
            Err(rustls::TLSError::WebPKIError(
                webpki::Error::ExtensionValueInvalid,
            ))
        }
    }
}
