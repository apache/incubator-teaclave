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

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate libc;
extern crate rand;
extern crate ring;
extern crate serde_json;
extern crate sgx_crypto;
extern crate sgx_types;
extern crate untrusted;
extern crate uuid;

use chrono::prelude::*;
use rand::{RngCore, SeedableRng};
use ring::signature;
use rocket::{data::ToByteUnit, http, response, Config};
use sgx_types::error::Quote3Error;
use sgx_types::types::*;

lazy_static! {
    static ref SIGNER: signature::RsaKeyPair = {
        let figment = Config::figment();
        let key_path = figment
            .extract_inner::<String>("attestation.key")
            .expect("key");
        let key = std::fs::read_to_string(key_path).unwrap();
        let der = pem::parse(key).unwrap().contents;
        signature::RsaKeyPair::from_pkcs8(&der).unwrap()
    };
    static ref REPORT_SIGNING_CERT: String = {
        let figment = Config::figment();
        let cert_path = figment
            .extract_inner::<String>("attestation.certs")
            .expect("certs");
        std::fs::read_to_string(cert_path).unwrap()
    };
}

#[link(name = "sgx_dcap_quoteverify")]
#[link(name = "sgx_dcap_ql")]
#[link(name = "sgx_urts")]
extern "C" {
    #[allow(improper_ctypes)]
    fn sgx_qv_verify_quote(
        p_quote: *const u8,
        quote_size: u32,
        p_quote_collateral: *const CQlQveCollateral,
        expiration_check_date: time_t,
        p_collateral_expiration_status: *mut u32,
        p_quote_verification_result: *mut QlQvResult,
        p_qve_report_info: *mut QlQeReportInfo,
        supplemental_data_size: u32,
        p_supplemental_data: *mut u8,
    ) -> Quote3Error;
}

enum QuoteVerificationResponse {
    BadRequest,
    InternalError,
    AcceptedRequest(QuoteVerificationResult),
}

struct QuoteVerificationResult {
    pub quote_status: QlQvResult,
    pub isv_enclave_quote: String,
}

impl QuoteVerificationResponse {
    fn accept(quote_status: QlQvResult, isv_enclave_quote: String) -> Self {
        Self::AcceptedRequest(QuoteVerificationResult {
            quote_status,
            isv_enclave_quote,
        })
    }
}

/// Convert SGX QL QV Result to str, try best to match the string defined in IAS
/// quote status APIs.
fn to_report(rst: &QlQvResult) -> &str {
    use QlQvResult::*;
    match rst {
        Ok => "OK",
        ConfigNeeded => "CONFIGURATION_NEEDED",
        OutOfDate => "OUT_OF_DATE",
        OutOfDateConfigNeeded => "OUT_OF_DATE_CONFIGURATION_NEEDED",
        InvalidSignature => "SIGNATURE_INVALID",
        Revoked => "KEY_REVOKED",
        Unspecified => "UNSPECIFIED",
        SWHardeningNeeded => "SW_HARDENING_NEEDED",
        ConfigAndSWHardeningNeeded => "CONFIGURATION_AND_SW_HARDENING_NEEDED",
        Max => panic!(),
    }
}

impl QuoteVerificationResult {
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "id": uuid::Uuid::new_v4().to_simple().to_string(),
            "version": 4,
            "timestamp": Utc::now().format("%Y-%m-%dT%H:%M:%S%.f").to_string(),
            "isvEnclaveQuoteStatus": to_report(&self.quote_status),
            "isvEnclaveQuoteBody": self.isv_enclave_quote,
        })
        .to_string()
    }
}

impl<'r> response::Responder<'r, 'static> for QuoteVerificationResponse {
    fn respond_to(self, _: &rocket::Request) -> response::Result<'static> {
        match self {
            Self::BadRequest => response::Result::Err(http::Status::BadRequest),
            Self::InternalError => response::Result::Err(http::Status::InternalServerError),
            Self::AcceptedRequest(qvr) => {
                let payload = qvr.to_json();
                let mut signature = vec![0; SIGNER.public_modulus_len()];
                let rng = ring::rand::SystemRandom::new();
                SIGNER
                    .sign(
                        &signature::RSA_PKCS1_SHA256,
                        &rng,
                        payload.as_bytes(),
                        &mut signature,
                    )
                    .unwrap();
                response::Response::build()
                    .header(http::ContentType::JSON)
                    .header(http::Header::new(
                        http::hyper::header::CONNECTION.as_str(),
                        "close",
                    ))
                    .raw_header(
                        "X-DCAPReport-Signing-Certificate",
                        percent_encoding::utf8_percent_encode(
                            &REPORT_SIGNING_CERT,
                            percent_encoding::NON_ALPHANUMERIC,
                        ),
                    )
                    .raw_header("X-DCAPReport-Signature", base64::encode(&signature))
                    .sized_body(payload.len(), std::io::Cursor::new(payload))
                    .ok()
            }
        }
    }
}

lazy_static! {
    static ref GLOBAL_MUTEX: std::sync::Mutex<i32> = std::sync::Mutex::new(0i32);
}

#[post(
    "/sgx/dev/attestation/v4/report",
    format = "application/json",
    data = "<request>"
)]
async fn verify_quote(request: rocket::Data<'_>) -> QuoteVerificationResponse {
    let bytes = request.open(1.megabytes()).into_bytes().await.unwrap();
    if !bytes.is_complete() {
        eprintln!("there are bytes remaining in the stream");
        return QuoteVerificationResponse::BadRequest;
    }

    let v = match serde_json::from_slice::<serde_json::Value>(&bytes) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("bad request {:?}", e);
            return QuoteVerificationResponse::BadRequest;
        }
    };

    if let serde_json::Value::String(base64_quote) = &v["isvEnclaveQuote"] {
        let quote = match base64::decode(base64_quote) {
            Ok(v) => v,
            Err(_) => return QuoteVerificationResponse::BadRequest,
        };
        let mut collateral_exp_status = 1u32;
        let mut quote_verification_result = QlQvResult::Unspecified;
        let mut qve_report_info = QlQeReportInfo::default();

        let mut nonce = QuoteNonce::default();
        let mut rng = rand::rngs::StdRng::from_entropy();
        rng.fill_bytes(&mut nonce.rand);
        qve_report_info.nonce = nonce;
        let mut expiration_check_date: time_t = 0;

        let ret = {
            let _lock = GLOBAL_MUTEX.lock();
            unsafe {
                sgx_qv_verify_quote(
                    quote.as_ptr(),
                    quote.len() as _,
                    std::ptr::null() as _,
                    libc::time(&mut expiration_check_date),
                    &mut collateral_exp_status as _,
                    &mut quote_verification_result as _,
                    &mut qve_report_info as _,
                    0,
                    std::ptr::null_mut(),
                )
            }
        };

        if ret != Quote3Error::Success {
            eprintln!("sgx_qv_verify_quote failed: {:?}", ret);
            return QuoteVerificationResponse::BadRequest;
        };

        if collateral_exp_status != 0 {
            eprintln!("collateral_exp_status failed: {:?}", collateral_exp_status);
            return QuoteVerificationResponse::BadRequest;
        }

        let mut sha256 = sgx_crypto::sha::Sha256::new().unwrap();
        sha256.update(&nonce.rand).unwrap();
        sha256.update(quote.as_slice()).unwrap();
        sha256.update(&expiration_check_date).unwrap();
        sha256.update(&collateral_exp_status).unwrap();
        sha256.update(&(quote_verification_result as u32)).unwrap();
        let sha256_hash = sha256.finalize().unwrap();

        // This check isn't quote necessary if we are verifying the nonce in
        // an untrusted environment
        if sha256_hash != qve_report_info.qe_report.body.report_data.d[..32]
            || [0u8; 32] != qve_report_info.qe_report.body.report_data.d[32..]
        {
            // Something wrong with out SW stack, probably compromised
            return QuoteVerificationResponse::InternalError;
        }

        // strip off signature data; client won't need this
        let quote_body = base64::encode(&quote[..432]);
        QuoteVerificationResponse::accept(quote_verification_result, quote_body)
    } else {
        QuoteVerificationResponse::BadRequest
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![verify_quote])
}
