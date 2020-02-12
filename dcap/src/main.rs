#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate libc;
extern crate rand;
extern crate ring;
extern crate serde_json;
extern crate sgx_types;
extern crate sgx_ucrypto;
extern crate untrusted;
extern crate uuid;

use chrono::prelude::*;
use rand::{RngCore, SeedableRng};
use ring::signature;
use rocket::{http, response};
use sgx_types::*;

const REPORT_SIGNING_CERT: &str = include_str!("../../keys/dcap_server_cert.pem");
const REPORT_SIGNING_KEY: &str = include_str!("../../keys/dcap_server_key.pem");

lazy_static! {
    static ref SIGNER: signature::RsaKeyPair = {
        let der = pem::parse(REPORT_SIGNING_KEY).unwrap().contents;
        signature::RsaKeyPair::from_pkcs8(&der).unwrap()
    };
}

#[link(name = "dcap_quoteverify")]
#[link(name = "sgx_dcap_ql")]
#[link(name = "sgx_urts")]
extern "C" {
    #[allow(improper_ctypes)]
    fn sgx_qv_verify_quote(
        p_quote: *const u8,
        quote_size: u32,
        p_quote_collateral: *const sgx_ql_qve_collateral_t,
        expiration_check_date: time_t,
        p_collateral_expiration_status: *mut u32,
        p_quote_verification_result: *mut sgx_ql_qv_result_t,
        p_qve_report_info: *mut sgx_ql_qe_report_info_t,
        supplemental_data_size: u32,
        p_supplemental_data: *mut u8,
    ) -> sgx_quote3_error_t;
}

enum QuoteVerificationResponse {
    BadRequest,
    InternalError,
    AcceptedRequest(QuoteVerificationResult),
}

struct QuoteVerificationResult {
    pub quote_status: sgx_ql_qv_result_t,
    pub isv_enclave_quote: String,
}

impl QuoteVerificationResponse {
    fn accept(quote_status: sgx_ql_qv_result_t, isv_enclave_quote: String) -> Self {
        Self::AcceptedRequest(QuoteVerificationResult {
            quote_status,
            isv_enclave_quote,
        })
    }
}

fn to_report(rst: sgx_ql_qv_result_t) -> &'static str {
    use sgx_ql_qv_result_t::*;
    match rst {
        SGX_QL_QV_RESULT_OK => "OK",
        SGX_QL_QV_RESULT_CONFIG_NEEDED => "CONFIGURATION_NEEDED",
        SGX_QL_QV_RESULT_OUT_OF_DATE => "OUT_OF_DATE",
        SGX_QL_QV_RESULT_OUT_OF_DATE_CONFIG_NEEDED => "OUT_OF_DATE_CONFIGURATION_NEEDED",
        SGX_QL_QV_RESULT_INVALID_SIGNATURE => "SIGNATURE_INVALID",
        SGX_QL_QV_RESULT_REVOKED => "KEY_REVOKED",
        _ => panic!(),
    }
}

impl QuoteVerificationResult {
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "id": uuid::Uuid::new_v4().to_simple().to_string(),
            "timestamp": Utc::now().format("%Y-%m-%dT%H:%M:%S%.f").to_string(),
            "isvEnclaveQuoteStatus": to_report(self.quote_status),
            "isvEnclaveQuoteBody": self.isv_enclave_quote,
        })
        .to_string()
    }
}

impl<'r> response::Responder<'r> for QuoteVerificationResponse {
    fn respond_to(self, _: &rocket::Request) -> response::Result<'r> {
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
                    .header(http::hyper::header::Connection::close())
                    .raw_header(
                        "X-DCAPReport-Signing-Certificate",
                        percent_encoding::utf8_percent_encode(
                            REPORT_SIGNING_CERT,
                            percent_encoding::NON_ALPHANUMERIC,
                        ),
                    )
                    .raw_header("X-DCAPReport-Signature", base64::encode(&signature))
                    .sized_body(std::io::Cursor::new(payload))
                    .ok()
            }
        }
    }
}

#[post(
    "/sgx/dev/attestation/v3/report",
    format = "application/json",
    data = "<request>"
)]
fn verify_quote(request: String) -> QuoteVerificationResponse {
    let v = match serde_json::from_str::<serde_json::Value>(&request) {
        Ok(v) => v,
        Err(_) => return QuoteVerificationResponse::BadRequest,
    };

    if let serde_json::Value::String(base64_quote) = &v["isvEnclaveQuote"] {
        let quote = match base64::decode(&base64_quote) {
            Ok(v) => v,
            Err(_) => return QuoteVerificationResponse::BadRequest,
        };
        let mut collateral_exp_status = 1u32;
        let mut quote_verification_result = sgx_ql_qv_result_t::SGX_QL_QV_RESULT_UNSPECIFIED;
        let mut qve_report_info = sgx_ql_qe_report_info_t::default();

        let mut nonce = sgx_quote_nonce_t::default();
        let mut rng = rand::rngs::StdRng::from_entropy();
        rng.fill_bytes(&mut nonce.rand);
        qve_report_info.nonce = nonce;
        let mut expiration_check_date: time_t = 0;
        let ret = unsafe {
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
        };

        if ret != sgx_quote3_error_t::SGX_QL_SUCCESS {
            eprintln!("sgx_qv_verify_quote fialed: {:?}", ret);
            return QuoteVerificationResponse::BadRequest;
        };

        if collateral_exp_status != 0 {
            eprintln!("collateral_exp_status fialed: {:?}", collateral_exp_status);
            return QuoteVerificationResponse::BadRequest;
        }

        let sha256 = sgx_ucrypto::SgxShaHandle::new();
        sha256.init().unwrap();
        sha256.update_msg(&nonce.rand).unwrap();
        sha256.update_slice(&quote.as_slice()).unwrap();
        sha256.update_msg(&expiration_check_date).unwrap();
        sha256.update_msg(&collateral_exp_status).unwrap();
        sha256
            .update_msg(&(quote_verification_result as u32))
            .unwrap();

        // This check isn't quote necessary if we are verifying the nonce in
        // an untrusted environment
        if sha256.get_hash().unwrap() != qve_report_info.qe_report.body.report_data.d[..32]
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

fn main() {
    rocket::ignite().mount("/", routes![verify_quote]).launch();
}
