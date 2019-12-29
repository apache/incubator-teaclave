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

use base64;
use bit_vec;
use chrono;
use httparse;
use num_bigint;
use rustls;
use webpki;
use webpki_roots;
use yasna;

use sgx_rand::os::SgxRng;
use sgx_rand::Rng;
use sgx_tcrypto::*;
use sgx_tse::*;
use sgx_types::*;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::ptr;
use std::sync::{Arc, SgxRwLock};
use std::time::{self, SystemTime};
use std::untrusted::time::SystemTimeEx;

use lazy_static::lazy_static;

use super::fail::MayfailTrace;
use crate::{Error, ErrorKind, Result};

use crate::config::runtime_config;
use teaclave_utils;

pub const CERT_VALID_DAYS: i64 = 90i64;

extern "C" {
    fn ocall_sgx_init_quote(
        p_retval: *mut sgx_status_t,
        p_target_info: *mut sgx_target_info_t,
        p_gid: *mut sgx_epid_group_id_t,
    ) -> sgx_status_t;

    fn ocall_sgx_get_ias_socket(p_retval: *mut i32) -> sgx_status_t;

    fn ocall_sgx_calc_quote_size(
        p_retval: *mut sgx_status_t,
        p_sig_rl: *const u8,
        sig_rl_size: u32,
        p_quote_size: *mut u32,
    ) -> sgx_status_t;

    fn ocall_sgx_get_quote(
        p_retval: *mut sgx_status_t,
        p_report: *const sgx_report_t,
        quote_type: sgx_quote_sign_type_t,
        p_spid: *const sgx_spid_t,
        p_nonce: *const sgx_quote_nonce_t,
        p_sig_rl: *const u8,
        sig_rl_size: u32,
        p_qe_report: *mut sgx_report_t,
        p_quote: *mut u8,
        quote_size: u32,
    ) -> sgx_status_t;
}

lazy_static! {
    static ref RACACHE: SgxRwLock<RACache> = {
        SgxRwLock::new(RACache {
            ra_credential: RACredential::default(),
            gen_time: SystemTime::UNIX_EPOCH,
            validity: time::Duration::from_secs(0),
        })
    };

    static ref IAS_CLIENT_CONFIG: Arc<rustls::ClientConfig> = {
        let mut config = rustls::ClientConfig::new();

        // We trust known CA
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

        Arc::new(config)
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
        let key_pair = Secp256k1KeyPair::new()?;
        let report = create_attestation_report(&key_pair.pub_k)?;
        let payload = [report.report, report.signature, report.certificate].join("|");
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

struct Secp256k1KeyPair {
    prv_k: sgx_ec256_private_t,
    pub_k: sgx_ec256_public_t,
}

impl Secp256k1KeyPair {
    fn new() -> Result<Self> {
        let ecc_handle = SgxEccHandle::new();
        ecc_handle.open()?;
        let (prv_k, pub_k) = ecc_handle.create_key_pair()?;
        ecc_handle.close()?;
        Ok(Self { prv_k, pub_k })
    }

    pub fn private_key_into_der(&self) -> Vec<u8> {
        use bit_vec::BitVec;
        use yasna::construct_der;
        use yasna::models::ObjectIdentifier;
        use yasna::Tag;

        let ec_public_key_oid = ObjectIdentifier::from_slice(&[1, 2, 840, 10045, 2, 1]);
        let prime256v1_oid = ObjectIdentifier::from_slice(&[1, 2, 840, 10045, 3, 1, 7]);

        let pub_key_bytes = self.public_key_into_bytes();
        let prv_key_bytes = self.private_key_into_bytes();

        // Construct private key in DER.
        construct_der(|writer| {
            writer.write_sequence(|writer| {
                writer.next().write_u8(0);
                writer.next().write_sequence(|writer| {
                    writer.next().write_oid(&ec_public_key_oid);
                    writer.next().write_oid(&prime256v1_oid);
                });
                let inner_key_der = construct_der(|writer| {
                    writer.write_sequence(|writer| {
                        writer.next().write_u8(1);
                        writer.next().write_bytes(&prv_key_bytes);
                        writer.next().write_tagged(Tag::context(1), |writer| {
                            writer.write_bitvec(&BitVec::from_bytes(&pub_key_bytes));
                        });
                    });
                });
                writer.next().write_bytes(&inner_key_der);
            });
        })
    }

    pub fn create_cert_with_extension(
        &self,
        issuer: &str,
        subject: &str,
        payload: &[u8],
    ) -> Vec<u8> {
        use super::cert::*;
        use bit_vec::BitVec;
        use chrono::TimeZone;
        use num_bigint::BigUint;
        use std::time::UNIX_EPOCH;
        use yasna::construct_der;
        use yasna::models::ObjectIdentifier;
        use yasna::models::UTCTime;

        let ecdsa_with_sha256_oid = ObjectIdentifier::from_slice(&[1, 2, 840, 10045, 4, 3, 2]);
        let common_name_oid = ObjectIdentifier::from_slice(&[2, 5, 4, 3]);
        let ec_public_key_oid = ObjectIdentifier::from_slice(&[1, 2, 840, 10045, 2, 1]);
        let prime256v1_oid = ObjectIdentifier::from_slice(&[1, 2, 840, 10045, 3, 1, 7]);
        let comment_oid = ObjectIdentifier::from_slice(&[2, 16, 840, 1, 113730, 1, 13]);

        let pub_key_bytes = self.public_key_into_bytes();

        // UNIX_EPOCH is the earliest time stamp. This unwrap should constantly succeed.
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let issue_ts = chrono::Utc.timestamp(now.as_secs() as i64, 0);

        // This is guaranteed to be a valid duration.
        let expire = now + chrono::Duration::days(CERT_VALID_DAYS).to_std().unwrap();
        let expire_ts = chrono::Utc.timestamp(expire.as_secs() as i64, 0);

        // Construct certificate with payload in extension in DER.
        let tbs_cert_der = construct_der(|writer| {
            let version = 2i8;
            let serial = 1u8;
            let cert_sign_algo = asn1_seq!(ecdsa_with_sha256_oid.clone());
            let issuer = asn1_seq!(asn1_seq!(asn1_seq!(
                common_name_oid.clone(),
                issuer.to_owned()
            )));
            let valid_range = asn1_seq!(
                UTCTime::from_datetime(&issue_ts),
                UTCTime::from_datetime(&expire_ts),
            );
            let subject = asn1_seq!(asn1_seq!(asn1_seq!(
                common_name_oid.clone(),
                subject.to_string(),
            )));
            let pub_key = asn1_seq!(
                asn1_seq!(ec_public_key_oid, prime256v1_oid,),
                BitVec::from_bytes(&pub_key_bytes),
            );
            let sgx_ra_cert_ext = asn1_seq!(asn1_seq!(comment_oid, payload.to_owned()));
            let tbs_cert = asn1_seq!(
                version,
                serial,
                cert_sign_algo,
                issuer,
                valid_range,
                subject,
                pub_key,
                sgx_ra_cert_ext,
            );
            TbsCert::dump(writer, tbs_cert);
        });

        // There will be serious problems if this call fails. We might as well
        // panic in this case, thus unwrap()
        let ecc_handle = SgxEccHandle::new();
        ecc_handle.open().unwrap();

        let sig = ecc_handle
            .ecdsa_sign_slice(&tbs_cert_der.as_slice(), &self.prv_k)
            .unwrap();

        let sig_der = yasna::construct_der(|writer| {
            writer.write_sequence(|writer| {
                let mut sig_x = sig.x;
                sig_x.reverse();
                let mut sig_y = sig.y;
                sig_y.reverse();
                writer.next().write_biguint(&BigUint::from_slice(&sig_x));
                writer.next().write_biguint(&BigUint::from_slice(&sig_y));
            });
        });

        yasna::construct_der(|writer| {
            writer.write_sequence(|writer| {
                writer.next().write_der(&tbs_cert_der.as_slice());
                CertSignAlgo::dump(writer.next(), asn1_seq!(ecdsa_with_sha256_oid.clone()));
                writer
                    .next()
                    .write_bitvec(&BitVec::from_bytes(&sig_der.as_slice()));
            });
        })
    }

    fn public_key_into_bytes(&self) -> Vec<u8> {
        // The first byte must be 4, which indicates the uncompressed encoding.
        let mut pub_key_bytes: Vec<u8> = vec![4];
        pub_key_bytes.extend(self.pub_k.gx.iter().rev());
        pub_key_bytes.extend(self.pub_k.gy.iter().rev());
        pub_key_bytes
    }

    fn private_key_into_bytes(&self) -> Vec<u8> {
        let mut prv_key_bytes: Vec<u8> = vec![];
        prv_key_bytes.extend(self.prv_k.r.iter().rev());
        prv_key_bytes
    }
}

trait MayfailTraceForHttparseStatus<T> {
    fn to_mt_result(self: Self, file: &'static str, line: u32) -> Result<T>;
}

impl<T> MayfailTraceForHttparseStatus<T> for httparse::Status<T> {
    fn to_mt_result(self: Self, file: &'static str, line: u32) -> Result<T> {
        match self {
            httparse::Status::Complete(v) => Ok(v),
            httparse::Status::Partial => {
                debug!("error at {}:{}", file, line);
                Err(Error::unknown())
            }
        }
    }
}

pub const DEV_HOSTNAME: &str = "api.trustedservices.intel.com";
// pub const PROD_HOSTNAME: &'static str = "as.sgx.trustedservices.intel.com";
pub const SIGRL_SUFFIX: &str = "/sgx/dev/attestation/v3/sigrl/";
pub const REPORT_SUFFIX: &str = "/sgx/dev/attestation/v3/report";

fn sanitize_http_response(respp: &httparse::Response) -> Result<()> {
    if let Some(code) = respp.code {
        if code != 200 {
            error!("Intel IAS service returned invalid HTTP {}", code);
            Err(Error::from(ErrorKind::RAInternalError))
        } else {
            Ok(())
        }
    } else {
        error!("Intel IAS service returned invalid HTTP response");
        Err(Error::from(ErrorKind::RAInternalError))
    }
}

struct AttnReport {
    pub report: String,
    pub signature: String,
    pub certificate: String,
}

fn parse_response_attn_report(resp: &[u8]) -> Result<AttnReport> {
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut respp = httparse::Response::new(&mut headers);
    let result = respp.parse(resp);

    sanitize_http_response(&respp)?;

    let mut sig: Result<String> = Err(Error::from(ErrorKind::MissingValue));
    let mut sig_cert: Result<String> = Err(Error::from(ErrorKind::MissingValue));
    let mut attn_report: Result<String> = Err(Error::from(ErrorKind::MissingValue));

    for header in respp.headers {
        match header.name {
            "Content-Length" => {
                let len_num = mayfail! {
                    len_str =<< std::str::from_utf8(header.value);
                    n =<< len_str.parse::<u32>();
                    ret n
                };

                if len_num.unwrap_or(0) > 0 {
                    attn_report = mayfail! {
                        status =<< result;
                        header_len =<< status;
                        let resp_body = &resp[header_len..];
                        report_str =<< std::str::from_utf8(resp_body);
                        ret report_str.to_string()
                    }
                }
            }
            "X-IASReport-Signature" => {
                sig = mayfail! {
                    sig =<< std::str::from_utf8(header.value);
                    ret sig.to_string()
                }
            }
            "X-IASReport-Signing-Certificate" => {
                sig_cert = mayfail! {
                    cert_str =<< std::str::from_utf8(header.value);
                    // Remove %0A from cert, and only obtain the signing cert
                    let cert = cert_str.to_string().replace("%0A", "");
                    // We should get two concatenated PEM files at this step
                    decoded_cert =<< teaclave_utils::percent_decode(cert);
                    let cert_content: Vec<&str> = decoded_cert.split("-----").collect();
                    _ =<< if cert_content.len() != 9 { None } else { Some(()) };
                    ret cert_content[2].to_string()
                }
            }
            _ => (),
        }
    }

    mayfail! {
        ret_sig =<< sig;
        ret_sig_cert =<< sig_cert;
        ret_attn_report =<< attn_report;
        ret AttnReport {
            report: ret_attn_report,
            signature: ret_sig,
            certificate: ret_sig_cert,
        }
    }
}

fn parse_response_sigrl(resp: &[u8]) -> Result<Vec<u8>> {
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut respp = httparse::Response::new(&mut headers);
    let result = respp.parse(resp);

    sanitize_http_response(&respp)?;

    let len_num = mayfail! {
        header =<< respp.headers.iter().find(|&&header| header.name == "Content-Length");
        len_str =<< std::str::from_utf8(header.value);
        len_num =<< len_str.parse::<u32>();
        ret len_num
    };

    len_num.and_then(|len| -> Result<Vec<u8>> {
        if len == 0 {
            Ok(Vec::new())
        } else {
            mayfail! {
                status =<< result;
                header_len =<< status;
                let resp_body = &resp[header_len..];
                base64 =<< std::str::from_utf8(resp_body);
                decoded =<< base64::decode(base64);
                ret decoded
            }
        }
    })
}

fn talk_to_intel_ias(fd: c_int, req: String) -> Result<Vec<u8>> {
    mayfail! {
        dns_name =<< webpki::DNSNameRef::try_from_ascii_str(DEV_HOSTNAME);
        let mut sess = rustls::ClientSession::new(&IAS_CLIENT_CONFIG, dns_name);
        mut sock =<< TcpStream::new(fd);
        let mut tls = rustls::Stream::new(&mut sess, &mut sock);
        _ =<< tls.write_all(req.as_bytes());
        let mut plaintext = Vec::new();
        _ =<< tls.read_to_end(&mut plaintext);
        ret plaintext
    }
}

fn get_sigrl_from_intel(fd: c_int, gid: u32) -> Result<Vec<u8>> {
    let req = format!(
        "GET {}{:08x} HTTP/1.1\r\nHOST: {}\r\nOcp-Apim-Subscription-Key: {}\r\nConnection: Close\r\n\r\n",
        SIGRL_SUFFIX, gid, DEV_HOSTNAME, &runtime_config().env.ias_key
    );

    mayfail! {
        plaintext =<< talk_to_intel_ias(fd, req);
        sigrl =<< parse_response_sigrl(&plaintext);
        ret sigrl
    }
}

// TODO: support pse
fn get_report_from_intel(fd: c_int, quote: &[u8]) -> Result<AttnReport> {
    let encoded_quote = base64::encode(quote);
    let encoded_json = format!("{{\"isvEnclaveQuote\":\"{}\"}}\r\n", encoded_quote);

    let req = format!("POST {} HTTP/1.1\r\nHOST: {}\r\nOcp-Apim-Subscription-Key: {}\r\nConnection: Close\r\nContent-Length:{}\r\nContent-Type: application/json\r\n\r\n{}",
                           REPORT_SUFFIX,
                           DEV_HOSTNAME,
                           &runtime_config().env.ias_key,
                           encoded_json.len(),
                           encoded_json);

    let plaintext = talk_to_intel_ias(fd, req)?;

    parse_response_attn_report(&plaintext)
}

fn create_attestation_report(pub_k: &sgx_ec256_public_t) -> Result<AttnReport> {
    if cfg!(sgx_sim) {
        return Ok(AttnReport {
            report: String::from(""),
            signature: String::from(""),
            certificate: String::from(""),
        });
    }
    // Workflow:
    // (1) ocall to get the target_info structure (ti) and epid group id (eg)
    // (1.5) get sigrl
    // (2) call sgx_create_report with ti+data, produce an sgx_report_t
    // (3) ocall to sgx_get_quote to generate (*mut sgx-quote_t, uint32_t)

    // (1) get ti + eg
    let mut ti: sgx_target_info_t = sgx_target_info_t::default();
    let mut eg: sgx_epid_group_id_t = sgx_epid_group_id_t::default();
    let mut rt: sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;

    let res = unsafe {
        ocall_sgx_init_quote(
            &mut rt as *mut sgx_status_t,
            &mut ti as *mut sgx_target_info_t,
            &mut eg as *mut sgx_epid_group_id_t,
        )
    };

    if res != sgx_status_t::SGX_SUCCESS || rt != sgx_status_t::SGX_SUCCESS {
        return Err(Error::from(ErrorKind::OCallError));
    }

    let eg_num = u32::from_le_bytes(eg);

    // (1.5) get sigrl
    let mut ias_sock: i32 = -1i32;

    let mut sigrl_vec: Vec<u8> = Vec::new();
    let mut sigrl_acquired: bool = false;
    for _ in 0..3 {
        let res = unsafe { ocall_sgx_get_ias_socket(&mut ias_sock as *mut i32) };

        debug!("got ias_sock = {}", ias_sock);

        if res != sgx_status_t::SGX_SUCCESS || ias_sock < 0 {
            return Err(Error::from(ErrorKind::OCallError));
        }

        // Now sigrl_vec is the revocation list, a vec<u8>
        match get_sigrl_from_intel(ias_sock, eg_num) {
            Ok(v) => {
                sigrl_vec = v;
                sigrl_acquired = true;
                break;
            }
            Err(_) => {
                debug!("get sirl failed, retry...");
            }
        }
    }

    if !sigrl_acquired {
        debug!("Cannot acquire sigrl from Intel for three times");
        return Err(Error::unknown());
    }

    // (2) Generate the report
    // Fill ecc256 public key into report_data
    let mut report_data: sgx_report_data_t = sgx_report_data_t::default();
    let mut pub_k_gx = pub_k.gx;
    pub_k_gx.reverse();
    let mut pub_k_gy = pub_k.gy;
    pub_k_gy.reverse();
    report_data.d[..32].clone_from_slice(&pub_k_gx);
    report_data.d[32..].clone_from_slice(&pub_k_gy);

    let rep = mayfail! {
        rep =<< rsgx_create_report(&ti, &report_data);
        ret rep
    }?;

    let mut quote_nonce = sgx_quote_nonce_t { rand: [0; 16] };
    let mut os_rng = mayfail! {
        rng =<< SgxRng::new();
        ret rng
    }?;

    os_rng.fill_bytes(&mut quote_nonce.rand);
    let mut qe_report = sgx_report_t::default();

    // (3) Generate the quote
    // Args:
    //       1. sigrl: ptr + len
    //       2. report: ptr 432bytes
    //       3. linkable: u32, unlinkable=0, linkable=1
    //       4. spid: sgx_spid_t ptr 16bytes
    //       5. sgx_quote_nonce_t ptr 16bytes
    //       6. p_sig_rl + sigrl size ( same to sigrl)
    //       7. [out]p_qe_report need further check
    //       8. [out]p_quote
    //       9. quote_size
    let (p_sigrl, sigrl_len) = if sigrl_vec.is_empty() {
        (ptr::null(), 0)
    } else {
        (sigrl_vec.as_ptr(), sigrl_vec.len() as u32)
    };
    let p_report = &rep as *const sgx_report_t;
    let quote_type = sgx_quote_sign_type_t::SGX_LINKABLE_SIGNATURE;
    let spid: sgx_spid_t = teaclave_utils::decode_spid(&runtime_config().env.ias_spid)?;
    let p_spid = &spid as *const sgx_spid_t;
    let p_nonce = &quote_nonce as *const sgx_quote_nonce_t;
    let p_qe_report = &mut qe_report as *mut sgx_report_t;
    let mut quote_len: u32 = 0;

    let res =
        unsafe { ocall_sgx_calc_quote_size(&mut rt as _, p_sigrl, sigrl_len, &mut quote_len as _) };

    if res != sgx_status_t::SGX_SUCCESS || rt != sgx_status_t::SGX_SUCCESS {
        return Err(Error::from(ErrorKind::OCallError));
    }

    let mut quote = vec![0; quote_len as usize];
    let p_quote = quote.as_mut_ptr();

    let res = unsafe {
        ocall_sgx_get_quote(
            &mut rt as _,
            p_report,
            quote_type,
            p_spid,
            p_nonce,
            p_sigrl,
            sigrl_len,
            p_qe_report,
            p_quote,
            quote_len,
        )
    };

    if res != sgx_status_t::SGX_SUCCESS || rt != sgx_status_t::SGX_SUCCESS {
        return Err(Error::from(ErrorKind::OCallError));
    }

    // Perform a check on qe_report to verify if the qe_report is valid
    rsgx_verify_report(&qe_report).to_mt_result(file!(), line!())?;

    // Check if the qe_report is produced on the same platform
    if ti.mr_enclave.m != qe_report.body.mr_enclave.m
        || ti.attributes.flags != qe_report.body.attributes.flags
        || ti.attributes.xfrm != qe_report.body.attributes.xfrm
    {
        return Err(Error::unknown());
    }

    // Check qe_report to defend against replay attack
    // The purpose of p_qe_report is for the ISV enclave to confirm the QUOTE
    // it received is not modified by the untrusted SW stack, and not a replay.
    // The implementation in QE is to generate a REPORT targeting the ISV
    // enclave (target info from p_report) , with the lower 32Bytes in
    // report.data = SHA256(p_nonce||p_quote). The ISV enclave can verify the
    // p_qe_report and report.data to confirm the QUOTE has not be modified and
    // is not a replay. It is optional.
    let mut rhs_vec: Vec<u8> = quote_nonce.rand.to_vec();
    rhs_vec.extend(&quote);
    let rhs_hash = rsgx_sha256_slice(&rhs_vec).to_mt_result(file!(), line!())?;
    let lhs_hash = &qe_report.body.report_data.d[..32];
    if rhs_hash != lhs_hash {
        return Err(Error::unknown());
    }

    let res = unsafe { ocall_sgx_get_ias_socket(&mut ias_sock as _) };

    if res != sgx_status_t::SGX_SUCCESS || ias_sock < 0 {
        return Err(Error::from(ErrorKind::OCallError));
    }

    get_report_from_intel(ias_sock, &quote)
}
