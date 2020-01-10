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

#![allow(clippy::redundant_closure)]

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use anyhow::{Error, Result};
use chrono::DateTime;
use rustls;
use serde_json;
use serde_json::Value;
use std::convert::TryFrom;
use std::time::*;

#[cfg(feature = "mesalock_sgx")]
use std::untrusted::time::SystemTimeEx;

use uuid::Uuid;

type SignatureAlgorithms = &'static [&'static webpki::SignatureAlgorithm];
static SUPPORTED_SIG_ALGS: SignatureAlgorithms = &[
    &webpki::ECDSA_P256_SHA256,
    &webpki::ECDSA_P256_SHA384,
    &webpki::ECDSA_P384_SHA256,
    &webpki::ECDSA_P384_SHA384,
    &webpki::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
    &webpki::RSA_PSS_2048_8192_SHA384_LEGACY_KEY,
    &webpki::RSA_PSS_2048_8192_SHA512_LEGACY_KEY,
    &webpki::RSA_PKCS1_2048_8192_SHA256,
    &webpki::RSA_PKCS1_2048_8192_SHA384,
    &webpki::RSA_PKCS1_2048_8192_SHA512,
    &webpki::RSA_PKCS1_3072_8192_SHA384,
];

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CertVerificationError {
    #[error("Invalid cert format")]
    InvalidCertFormat,
    #[error("Bad attestation report")]
    BadAttnReport,
    #[error("Webpki failure")]
    WebpkiFailure,
}

pub struct SgxReport {
    pub cpu_svn: [u8; 16],
    pub misc_select: u32,
    pub attributes: [u8; 16],
    pub mr_enclave: [u8; 32],
    pub mr_signer: [u8; 32],
    pub isv_prod_id: u16,
    pub isv_svn: u16,
    pub report_data: [u8; 64],
}

pub enum SgxQuoteVersion {
    V1,
    V2,
}

pub enum SgxQuoteSigType {
    Unlinkable,
    Linkable,
}

#[derive(PartialEq, Debug)]
pub enum SgxQuoteStatus {
    OK,
    GroupOutOfDate,
    ConfigurationNeeded,
    UnknownBadStatus,
}

impl From<&str> for SgxQuoteStatus {
    fn from(status: &str) -> Self {
        match status {
            "OK" => SgxQuoteStatus::OK,
            "GROUP_OUT_OF_DATE" => SgxQuoteStatus::GroupOutOfDate,
            "CONFIGURATION_NEEDED" => SgxQuoteStatus::ConfigurationNeeded,
            _ => SgxQuoteStatus::UnknownBadStatus,
        }
    }
}

pub struct SgxQuoteBody {
    pub version: SgxQuoteVersion,
    pub signature_type: SgxQuoteSigType,
    pub gid: u32,
    pub isv_svn_qe: u16,
    pub isv_svn_pce: u16,
    pub qe_vendor_id: Uuid,
    pub user_data: [u8; 20],
    pub report_body: SgxReport,
}

impl SgxQuoteBody {
    // TODO: A Result should be returned instead of Option
    fn parse_from<'a>(bytes: &'a [u8]) -> Option<Self> {
        let mut pos: usize = 0;
        // TODO: It is really unnecessary to construct a Vec<u8> each time.
        // Try to optimize this.
        let mut take = |n: usize| -> Option<&'a [u8]> {
            if n > 0 && bytes.len() >= pos + n {
                let ret = Some(&bytes[pos..pos + n]);
                pos += n;
                ret
            } else {
                None
            }
        };

        // off 0, size 2
        let version = match u16::from_le_bytes(<[u8; 2]>::try_from(take(2)?).ok()?) {
            1 => SgxQuoteVersion::V1,
            2 => SgxQuoteVersion::V2,
            _ => return None,
        };

        // off 2, size 2
        let signature_type = match u16::from_le_bytes(<[u8; 2]>::try_from(take(2)?).ok()?) {
            0 => SgxQuoteSigType::Unlinkable,
            1 => SgxQuoteSigType::Linkable,
            _ => return None,
        };

        // off 4, size 4
        let gid = u32::from_le_bytes(<[u8; 4]>::try_from(take(4)?).ok()?);

        // off 8, size 2
        let isv_svn_qe = u16::from_le_bytes(<[u8; 2]>::try_from(take(2)?).ok()?);

        // off 10, size 2
        let isv_svn_pce = u16::from_le_bytes(<[u8; 2]>::try_from(take(2)?).ok()?);

        // off 12, size 16
        let qe_vendor_id_raw = <[u8; 16]>::try_from(take(16)?).ok()?;
        let qe_vendor_id = Uuid::from_slice(&qe_vendor_id_raw).ok()?;

        // off 28, size 20
        let user_data = <[u8; 20]>::try_from(take(20)?).ok()?;

        // off 48, size 16
        let cpu_svn = <[u8; 16]>::try_from(take(16)?).ok()?;

        // off 64, size 4
        let misc_select = u32::from_le_bytes(<[u8; 4]>::try_from(take(4)?).ok()?);

        // off 68, size 28
        let _reserved = take(28)?;

        // off 96, size 16
        let attributes = <[u8; 16]>::try_from(take(16)?).ok()?;

        // off 112, size 32
        let mr_enclave = <[u8; 32]>::try_from(take(32)?).ok()?;

        // off 144, size 32
        let _reserved = take(32)?;

        // off 176, size 32
        let mr_signer = <[u8; 32]>::try_from(take(32)?).ok()?;

        // off 208, size 96
        let _reserved = take(96)?;

        // off 304, size 2
        let isv_prod_id = u16::from_le_bytes(<[u8; 2]>::try_from(take(2)?).ok()?);

        // off 306, size 2
        let isv_svn = u16::from_le_bytes(<[u8; 2]>::try_from(take(2)?).ok()?);

        // off 308, size 60
        let _reserved = take(60)?;

        // off 368, size 64
        let mut report_data = [0u8; 64];
        let _report_data = take(64)?;
        let mut _it = _report_data.iter();
        for i in report_data.iter_mut() {
            *i = *_it.next()?;
        }

        if pos != bytes.len() {
            return None;
        }

        Some(Self {
            version,
            signature_type,
            gid,
            isv_svn_qe,
            isv_svn_pce,
            qe_vendor_id,
            user_data,
            report_body: SgxReport {
                cpu_svn,
                misc_select,
                attributes,
                mr_enclave,
                mr_signer,
                isv_prod_id,
                isv_svn,
                report_data,
            },
        })
    }
}

pub struct SgxQuote {
    pub freshness: Duration,
    pub status: SgxQuoteStatus,
    pub body: SgxQuoteBody,
}

impl SgxQuote {
    pub fn extract_verified_quote(cert_der: &[u8], ias_report_ca_cert: &[u8]) -> Result<SgxQuote> {
        // Before we reach here, Webpki already verifed the cert is properly signed
        use super::cert::*;

        let x509 = yasna::parse_der(cert_der, |reader| X509::load(reader))
            .map_err(|_| CertVerificationError::InvalidCertFormat)?;

        let tbs_cert: <TbsCert as Asn1Ty>::ValueTy = x509.0;

        let pub_key: <PubKey as Asn1Ty>::ValueTy = ((((((tbs_cert.1).1).1).1).1).1).0;
        let pub_k = (pub_key.1).0;

        let sgx_ra_cert_ext: <SgxRaCertExt as Asn1Ty>::ValueTy =
            (((((((tbs_cert.1).1).1).1).1).1).1).0;

        let payload: Vec<u8> = ((sgx_ra_cert_ext.0).1).0;

        // Extract each field
        let mut iter = payload.split(|x| *x == 0x7C);
        let attn_report_raw = iter
            .next()
            .ok_or_else(|| Error::new(CertVerificationError::InvalidCertFormat))?;
        let sig_raw = iter
            .next()
            .ok_or_else(|| Error::new(CertVerificationError::InvalidCertFormat))?;
        let sig = base64::decode(&sig_raw)?;
        let sig_cert_raw = iter
            .next()
            .ok_or_else(|| Error::new(CertVerificationError::InvalidCertFormat))?;
        let sig_cert_dec = base64::decode_config(&sig_cert_raw, base64::STANDARD)?;

        let sig_cert = webpki::EndEntityCert::from(&sig_cert_dec)
            .map_err(|_| CertVerificationError::InvalidCertFormat)?;

        // Verify if the signing cert is issued by Intel CA
        // let mut ias_ca_stripped = ias_report_ca_cert.to_vec();
        // ias_ca_stripped.retain(|&x| x != 0x0d && x != 0x0a);
        // let head_len = "-----BEGIN CERTIFICATE-----".len();
        // let tail_len = "-----END CERTIFICATE-----".len();
        // let full_len = ias_ca_stripped.len();
        // let ias_ca_core: &[u8] = &ias_ca_stripped[head_len..full_len - tail_len];
        // let ias_cert_dec = base64::decode_config(ias_ca_core, base64::STANDARD)
        //     .map_err(|_| CertVerificationError::InvalidCertFormat)?;

        // let mut ca_reader = BufReader::new(ias_report_ca_cert);

        let mut root_store = rustls::RootCertStore::empty();
        root_store
            .add(&rustls::Certificate(ias_report_ca_cert.to_vec()))
            .expect("Failed to add CA");

        let trust_anchors: Vec<webpki::TrustAnchor> = root_store
            .roots
            .iter()
            .map(|cert| cert.to_trust_anchor())
            .collect();

        let chain: Vec<&[u8]> = vec![ias_report_ca_cert];

        let now_func = webpki::Time::try_from(SystemTime::now())
            .map_err(|_| CertVerificationError::WebpkiFailure)?;

        sig_cert
            .verify_is_valid_tls_server_cert(
                SUPPORTED_SIG_ALGS,
                &webpki::TLSServerTrustAnchors(&trust_anchors),
                &chain,
                now_func,
            )
            .map_err(|_| CertVerificationError::WebpkiFailure)?;

        // Verify the signature against the signing cert
        sig_cert
            .verify_signature(&webpki::RSA_PKCS1_2048_8192_SHA256, &attn_report_raw, &sig)
            .map_err(|_| CertVerificationError::WebpkiFailure)?;

        // Verify attestation report
        let attn_report: Value = serde_json::from_slice(attn_report_raw)
            .map_err(|_| CertVerificationError::BadAttnReport)?;

        // 1. Check timestamp is within 24H (90day is recommended by Intel)
        let quote_freshness = {
            let time = attn_report["timestamp"]
                .as_str()
                .ok_or_else(|| Error::new(CertVerificationError::BadAttnReport))?;
            let time_fixed = String::from(time) + "+0000";
            let date_time = DateTime::parse_from_str(&time_fixed, "%Y-%m-%dT%H:%M:%S%.f%z")?;
            let ts = date_time.naive_utc();
            let now = DateTime::<chrono::offset::Utc>::from(SystemTime::now()).naive_utc();
            u64::try_from((now - ts).num_seconds())?
        };

        // 2. Get quote status
        let quote_status = {
            let status_string = attn_report["isvEnclaveQuoteStatus"]
                .as_str()
                .ok_or_else(|| Error::new(CertVerificationError::BadAttnReport))?;

            SgxQuoteStatus::from(status_string)
        };

        // 3. Get quote body
        let quote_body = {
            let quote_encoded = attn_report["isvEnclaveQuoteBody"]
                .as_str()
                .ok_or_else(|| Error::new(CertVerificationError::BadAttnReport))?;
            let quote_raw = base64::decode(&(quote_encoded.as_bytes()))?;
            SgxQuoteBody::parse_from(quote_raw.as_slice())
                .ok_or_else(|| Error::new(CertVerificationError::BadAttnReport))?
        };

        let raw_pub_k = pub_k.to_bytes();

        // According to RFC 5480 `Elliptic Curve Cryptography Subject Public Key Information',
        // SEC 2.2:
        // ``The first octet of the OCTET STRING indicates whether the key is
        // compressed or uncompressed.  The uncompressed form is indicated
        // by 0x04 and the compressed form is indicated by either 0x02 or
        // 0x03 (see 2.3.3 in [SEC1]).  The public key MUST be rejected if
        // any other value is included in the first octet.''
        //
        // We only accept the uncompressed form here.
        let is_uncompressed = raw_pub_k[0] == 4;
        let pub_k = &raw_pub_k.as_slice()[1..];
        if !is_uncompressed || pub_k != &quote_body.report_body.report_data[..] {
            return Err(Error::new(CertVerificationError::BadAttnReport));
        }

        Ok(SgxQuote {
            freshness: std::time::Duration::from_secs(quote_freshness),
            status: quote_status,
            body: quote_body,
        })
    }
}
