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

//! This module provides types used to verify attestation reports.

use crate::report::AttestationReport;

use std::vec::Vec;

use log::{debug, error};
use teaclave_types::EnclaveAttr;

/// User defined verification function to further verify the attestation report.
pub type AttestationReportVerificationFn = fn(&AttestationReport) -> bool;

/// Type used to verify attestation reports (this can be set as a certificate
/// verifier in `rustls::ClientConfig`).
#[derive(Clone)]
pub struct AttestationReportVerifier {
    /// Valid enclave attributes (only enclaves with attributes in this vector
    /// will be accepted).
    pub accepted_enclave_attrs: Vec<EnclaveAttr>,
    /// Root certificate of the attestation service provider (e.g., IAS).
    pub root_ca: Vec<u8>,
    /// User defined function to verify the attestation report.
    pub verifier: AttestationReportVerificationFn,
}

/// Checks if he quote's status is not `UnknownBadStatus`
pub fn universal_quote_verifier(report: &AttestationReport) -> bool {
    debug!("report.sgx_quote_status: {:?}", report.sgx_quote_status);
    report.sgx_quote_status != crate::report::SgxQuoteStatus::UnknownBadStatus
}

impl AttestationReportVerifier {
    pub fn new(
        accepted_enclave_attrs: Vec<EnclaveAttr>,
        root_ca: &[u8],
        verifier: AttestationReportVerificationFn,
    ) -> Self {
        Self {
            accepted_enclave_attrs,
            root_ca: root_ca.to_vec(),
            verifier,
        }
    }

    /// Verify whether the `MR_SIGNER` and `MR_ENCLAVE` in the attestation report is
    /// accepted by us, which are defined in `accepted_enclave_attrs`.
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

    /// Verify TLS certificate.
    fn verify_cert(&self, certs: &[rustls::Certificate]) -> bool {
        debug!("verify cert");
        if cfg!(sgx_sim) {
            return true;
        }

        let report = match AttestationReport::from_cert(certs, &self.root_ca) {
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

impl rustls::client::ServerCertVerifier for AttestationReportVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp: &[u8],
        _now: std::time::SystemTime,
    ) -> std::result::Result<rustls::client::ServerCertVerified, rustls::Error> {
        // This call automatically verifies certificate signature
        debug!("verify server cert");
        if self.verify_cert(&[end_entity.to_owned()]) {
            Ok(rustls::client::ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::UnhandledCriticalExtension,
            ))
        }
    }
}

impl rustls::server::ClientCertVerifier for AttestationReportVerifier {
    fn offer_client_auth(&self) -> bool {
        // If test_mode is on, then disable TLS client authentication.
        !cfg!(test_mode)
    }

    fn client_auth_root_subjects(&self) -> &[rustls::DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _now: std::time::SystemTime,
    ) -> std::result::Result<rustls::server::ClientCertVerified, rustls::Error> {
        // This call automatically verifies certificate signature
        debug!("verify client cert");

        if self.verify_cert(&[end_entity.to_owned()]) {
            Ok(rustls::server::ClientCertVerified::assertion())
        } else {
            Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::UnhandledCriticalExtension,
            ))
        }
    }
}
