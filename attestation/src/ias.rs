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

use crate::AttestationError;
use anyhow::Error;
use anyhow::Result;
use anyhow::{anyhow, bail};
use log::{debug, trace};
use percent_encoding;
use serde::{Deserialize, Serialize};
use sgx_types::*;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::unix::io::FromRawFd;
use std::prelude::v1::*;
use std::sync::Arc;

#[cfg(feature = "mesalock_sgx")]
extern "C" {
    fn ocall_sgx_get_ias_socket(p_retval: *mut i32) -> sgx_status_t;
}

#[derive(Default, Serialize, Deserialize)]
pub struct IasReport {
    pub report: Vec<u8>,
    pub signature: Vec<u8>,
    pub signing_cert: Vec<u8>,
}

impl IasReport {
    #[cfg(feature = "mesalock_sgx")]
    pub fn new(
        pub_k: sgx_types::sgx_ec256_public_t,
        ias_key: &str,
        ias_spid: &str,
    ) -> anyhow::Result<Self> {
        use crate::platform;
        let (target_info, epid_group_id) = platform::init_sgx_quote()?;
        let mut ias_client = IasClient::new(ias_key);
        let sigrl = ias_client.get_sigrl(u32::from_le_bytes(epid_group_id))?;
        let sgx_report = platform::create_sgx_report(pub_k, target_info)?;
        let quote = platform::get_sgx_quote(&sigrl, sgx_report, target_info, ias_spid)?;
        let ias_report = ias_client.get_report(&quote)?;
        Ok(ias_report)
    }
}

pub struct IasClient {
    ias_key: String,
    ias_hostname: &'static str,
}

impl IasClient {
    pub fn new(ias_key: &str) -> Self {
        #[cfg(production)]
        let ias_hostname = "as.sgx.trustedservices.intel.com";
        #[cfg(not(production))]
        let ias_hostname = "api.trustedservices.intel.com";

        Self {
            ias_key: ias_key.to_owned(),
            ias_hostname,
        }
    }

    fn new_tls_stream(&self) -> Result<rustls::StreamOwned<rustls::ClientSession, TcpStream>> {
        let dns_name = webpki::DNSNameRef::try_from_ascii_str(self.ias_hostname)?;
        let mut config = rustls::ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        let client = rustls::ClientSession::new(&Arc::new(config), dns_name);
        let fd = get_ias_socket()?;
        let socket = unsafe { TcpStream::from_raw_fd(fd) };
        let stream = rustls::StreamOwned::new(client, socket);

        Ok(stream)
    }

    pub fn get_sigrl(&mut self, epid_group_id: u32) -> Result<Vec<u8>> {
        let sigrl_uri = format!("/sgx/dev/attestation/v3/sigrl/{:08x}", epid_group_id);
        let request = format!(
            "GET {} HTTP/1.1\r\n\
             HOST: {}\r\n\
             Ocp-Apim-Subscription-Key: {}\r\n\
             Connection: Close\r\n\r\n",
            sigrl_uri, self.ias_hostname, self.ias_key
        );

        let mut stream = self.new_tls_stream()?;
        stream.write_all(request.as_bytes())?;
        let mut response = Vec::new();
        stream.read_to_end(&mut response)?;

        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut http_response = httparse::Response::new(&mut headers);
        let header_len = match http_response
            .parse(&response)
            .map_err(|_| Error::new(AttestationError::IasError))?
        {
            httparse::Status::Complete(s) => s,
            _ => bail!(AttestationError::IasError),
        };

        let header_map = parse_headers(&http_response);

        if !header_map.contains_key("Content-Length")
            || header_map
                .get("Content-Length")
                .unwrap()
                .parse::<u32>()
                .unwrap_or(0)
                == 0
        {
            Ok(Vec::new())
        } else {
            let base64 = std::str::from_utf8(&response[header_len..])?;

            let decoded = base64::decode(base64)?;
            Ok(decoded)
        }
    }

    pub fn get_report(&mut self, quote: &[u8]) -> Result<IasReport> {
        debug!("get_report");
        let report_uri = "/sgx/dev/attestation/v3/report";
        let encoded_quote = base64::encode(quote);
        let encoded_json = format!("{{\"isvEnclaveQuote\":\"{}\"}}\r\n", encoded_quote);

        let request = format!(
            "POST {} HTTP/1.1\r\n\
             HOST: {}\r\n\
             Ocp-Apim-Subscription-Key: {}\r\n\
             Connection: Close\r\n\
             Content-Length: {}\r\n\
             Content-Type: application/json\r\n\r\n\
             {}",
            report_uri,
            self.ias_hostname,
            self.ias_key,
            encoded_json.len(),
            encoded_json
        );
        trace!("{}", request);

        let mut stream = self.new_tls_stream()?;
        stream.write_all(request.as_bytes())?;
        let mut response = Vec::new();
        stream.read_to_end(&mut response)?;

        trace!("{}", String::from_utf8_lossy(&response));

        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut http_response = httparse::Response::new(&mut headers);
        debug!("http_response.parse");
        let header_len = match http_response
            .parse(&response)
            .map_err(|_| Error::new(AttestationError::IasError))?
        {
            httparse::Status::Complete(s) => s,
            _ => bail!(AttestationError::IasError),
        };

        let header_map = parse_headers(&http_response);

        debug!("get_content_length");
        if !header_map.contains_key("Content-Length")
            || header_map
                .get("Content-Length")
                .unwrap()
                .parse::<u32>()
                .unwrap_or(0)
                == 0
        {
            bail!(AttestationError::IasError);
        }

        debug!("get_signature");
        let signature = header_map
            .get("X-IASReport-Signature")
            .ok_or_else(|| Error::new(AttestationError::IasError))?;
        let signature = base64::decode(signature)?;
        debug!("get_signing_cert");
        let signing_cert = {
            let cert_str = header_map
                .get("X-IASReport-Signing-Certificate")
                .ok_or_else(|| Error::new(AttestationError::IasError))?;
            let decoded_cert = percent_encoding::percent_decode_str(cert_str).decode_utf8()?;
            let certs = rustls::internal::pemfile::certs(&mut decoded_cert.as_bytes())
                .map_err(|_| anyhow!("pemfile error"))?;
            certs[0].0.clone()
        };

        let report = response[header_len..].to_vec();
        Ok(IasReport {
            report,
            signature,
            signing_cert,
        })
    }
}

fn parse_headers(resp: &httparse::Response) -> HashMap<String, String> {
    debug!("parse_headers");
    let mut header_map = HashMap::new();
    for h in resp.headers.iter() {
        header_map.insert(
            h.name.to_owned(),
            String::from_utf8_lossy(h.value).into_owned(),
        );
    }

    header_map
}

#[cfg(feature = "mesalock_sgx")]
fn get_ias_socket() -> Result<c_int> {
    debug!("get_ias_socket");
    let mut fd: c_int = -1;
    let res = unsafe { ocall_sgx_get_ias_socket(&mut fd as _) };

    if res != sgx_status_t::SGX_SUCCESS || fd < 0 {
        bail!(AttestationError::OCallError)
    } else {
        Ok(fd)
    }
}

#[cfg(not(feature = "mesalock_sgx"))]
fn get_ias_socket() -> Result<c_int> {
    use std::os::unix::io::IntoRawFd;
    let ias_addr = "api.trustedservices.intel.com:443";
    let stream = TcpStream::connect(ias_addr)?;
    Ok(stream.into_raw_fd())
}
