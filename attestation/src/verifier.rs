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

use crate::report::AttestationReport;
use log::{debug, error};
use std::vec::Vec;
use teaclave_types::EnclaveAttr;

#[derive(Clone)]
pub struct AttestationReportVerifier {
    pub accepted_enclave_attrs: Vec<EnclaveAttr>,
    pub root_ca: Vec<u8>,
    pub verifier: fn(&AttestationReport) -> bool,
}

pub fn universal_quote_verifier(report: &AttestationReport) -> bool {
    report.sgx_quote_status != crate::report::SgxQuoteStatus::UnknownBadStatus
}

impl AttestationReportVerifier {
    pub fn new(
        accepted_enclave_attrs: Vec<EnclaveAttr>,
        root_ca: &[u8],
        verifier: fn(&AttestationReport) -> bool,
    ) -> Self {
        Self {
            accepted_enclave_attrs,
            root_ca: root_ca.to_vec(),
            verifier,
        }
    }

    fn verify_measures(&self, attestation_report: &AttestationReport) -> bool {
        debug!("verify measures");
        let this_mr_signer = attestation_report
            .sgx_quote_body
            .isv_enclave_report
            .mr_signer;
        let this_mr_enclave = attestation_report
            .sgx_quote_body
            .isv_enclave_report
            .mr_enclave;

        self.accepted_enclave_attrs.iter().any(|a| {
            a.measurement.mr_signer == this_mr_signer && a.measurement.mr_enclave == this_mr_enclave
        })
    }

    fn verify_cert(&self, cert_der: &[u8]) -> bool {
        debug!("verify cert");
        if cfg!(sgx_sim) {
            return true;
        }

        let report = match AttestationReport::from_cert(&cert_der, &self.root_ca) {
            Ok(report) => report,
            Err(e) => {
                error!("cert verification error {:?}", e);
                return false;
            }
        };

        // Enclave measures are not tested in test mode since we have
        // a dedicated test enclave not known to production enclaves
        if cfg!(test_mode) {
            return (self.verifier)(&report);
        }

        self.verify_measures(&report) && (self.verifier)(&report)
    }
}

impl rustls::ServerCertVerifier for AttestationReportVerifier {
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

impl rustls::ClientCertVerifier for AttestationReportVerifier {
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
