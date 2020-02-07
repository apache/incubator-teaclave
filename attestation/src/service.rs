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

use crate::platform;
use crate::AttestationAlgorithm;
use crate::AttestationError;
use crate::AttestationServiceConfig;
use crate::EndorsedAttestationReport;
use anyhow::Error;
use anyhow::Result;
use anyhow::{anyhow, bail};
use log::{debug, trace};
use percent_encoding;
use sgx_types::*;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::unix::io::FromRawFd;
use std::prelude::v1::*;
use std::sync::Arc;

impl EndorsedAttestationReport {
    pub(crate) fn new(
        att_service_cfg: &AttestationServiceConfig,
        pub_k: sgx_types::sgx_ec256_public_t,
    ) -> anyhow::Result<Self> {
        let (mut ak_id, qe_target_info) = platform::init_sgx_quote()?;

        // For IAS-based attestation, we need to fill our SPID (obtained from Intel)
        // into the attestation key id. For DCAP-based attestation, SPID should be 0
        const SPID_OFFSET: usize = std::mem::size_of::<sgx_ql_att_key_id_t>();
        ak_id.att_key_id[SPID_OFFSET..(SPID_OFFSET + att_service_cfg.spid.id.len())]
            .clone_from_slice(&att_service_cfg.spid.id);

        let sgx_report = platform::create_sgx_isv_enclave_report(pub_k, qe_target_info)?;
        let quote = platform::get_sgx_quote(&ak_id, sgx_report)?;
        let as_report = get_report(
            &att_service_cfg.algo,
            &att_service_cfg.as_url,
            &att_service_cfg.api_key,
            &quote,
        )?;

        Ok(as_report)
    }
}

fn new_tls_stream(url: &url::Url) -> Result<rustls::StreamOwned<rustls::ClientSession, TcpStream>> {
    let dns_name = webpki::DNSNameRef::try_from_ascii_str(url.host_str().unwrap())?;
    let mut config = rustls::ClientConfig::new();
    config
        .root_store
        .add(&rustls::Certificate(
            include_bytes!("../../keys/dcap_root_ca_cert.der").to_vec(),
        ))
        .unwrap();
    config
        .root_store
        .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
    let client = rustls::ClientSession::new(&Arc::new(config), dns_name);
    let fd = platform::get_attestation_service_socket(&url.to_string())?;
    let socket = unsafe { TcpStream::from_raw_fd(fd) };
    let stream = rustls::StreamOwned::new(client, socket);

    Ok(stream)
}

fn get_report(
    algo: &AttestationAlgorithm,
    url: &url::Url,
    api_key: &str,
    quote: &[u8],
) -> Result<EndorsedAttestationReport> {
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
        url.host_str().unwrap(),
        api_key,
        encoded_json.len(),
        encoded_json
    );
    trace!("{}", request);

    let mut stream = new_tls_stream(url)?;
    stream.write_all(request.as_bytes())?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;

    trace!("{}", String::from_utf8_lossy(&response));

    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut http_response = httparse::Response::new(&mut headers);

    debug!("http_response.parse");
    let header_len = match http_response
        .parse(&response)
        .map_err(|_| Error::new(AttestationError::AttestationServiceError))?
    {
        httparse::Status::Complete(s) => s,
        _ => bail!(AttestationError::AttestationServiceError),
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
        bail!(AttestationError::AttestationServiceError);
    }

    debug!("get_signature");
    let signature_header = match algo {
        AttestationAlgorithm::SgxEpid => "X-IASReport-Signature",
        AttestationAlgorithm::SgxEcdsa => "X-DCAPReport-Signature",
    };
    let signature = header_map
        .get(signature_header)
        .ok_or_else(|| Error::new(AttestationError::AttestationServiceError))?;
    let signature = base64::decode(signature)?;

    debug!("get_signing_cert");
    let signing_cert_header = match algo {
        AttestationAlgorithm::SgxEpid => "X-IASReport-Signing-Certificate",
        AttestationAlgorithm::SgxEcdsa => "X-DCAPReport-Signing-Certificate",
    };
    let signing_cert = {
        let cert_str = header_map
            .get(signing_cert_header)
            .ok_or_else(|| Error::new(AttestationError::AttestationServiceError))?;
        let decoded_cert = percent_encoding::percent_decode_str(cert_str).decode_utf8()?;
        let certs = rustls::internal::pemfile::certs(&mut decoded_cert.as_bytes())
            .map_err(|_| anyhow!("pemfile error"))?;
        certs[0].0.clone()
    };

    debug!("return_report");
    let report = response[header_len..].to_vec();
    Ok(EndorsedAttestationReport {
        report,
        signature,
        signing_cert,
    })
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
