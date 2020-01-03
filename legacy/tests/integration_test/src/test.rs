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

use exitfailure::ExitFailure;
use failure::ResultExt;
use log::debug;
use serde_derive::Deserialize;

use regex::Regex;
use rustls::{ClientConfig, ClientSession, Stream};
use std::collections::BTreeMap;
use std::net::TcpStream;
use std::sync::Arc;
use webpki::DNSNameRef;

use crate::sendrecv::{recv_vec, send_vec};

#[derive(Debug, Deserialize)]
pub struct Endpoint {
    ip: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Step {
    name: String,
    endpoint: String, // name of Endpoint
    payload: String,
    expected: String,
}

#[derive(Debug, Deserialize)]
pub struct Case {
    name: String,
    endpoint: BTreeMap<String, Endpoint>,
    step: Vec<Step>,
}

impl Case {
    pub fn new_from_toml(data: &str) -> Result<Case, ExitFailure> {
        let case: Case = toml::from_str(&data)
            .with_context(|_| "could not parse the test case file".to_string())?;
        Ok(case)
    }
}

pub struct Runner {
    tls_ctx: Arc<ClientConfig>,
}

impl Runner {
    pub fn new() -> Runner {
        Runner {
            tls_ctx: Arc::new(get_tls_config()),
        }
    }

    pub fn run_test(&self, case: &Case) -> Result<(), ExitFailure> {
        let endpoint_map = &case.endpoint;
        println!("testing \"{}\" with {} step(s)", case.name, case.step.len());
        for step in case.step.iter() {
            match endpoint_map.get(&step.endpoint) {
                Some(endpoint) => match self.run_step(&endpoint, &step.payload, &step.expected) {
                    Ok(_) => {
                        println!("testing step {} ... \x1B[1;32mok\x1B[0m", step.name);
                    }
                    Err(e) => {
                        println!("testing step {} ... \x1B[1;31mfailed\x1B[0m", step.name,);
                        return Err(e);
                    }
                },
                None => {
                    return Err(failure::err_msg(format!(
                        "Step {} uses an invalid endpoint {}.",
                        step.name, step.endpoint
                    ))
                    .into());
                }
            }
        }
        Ok(())
    }

    fn run_step(
        &self,
        endpoint: &Endpoint,
        request: &str,
        expected_response: &str,
    ) -> Result<(), ExitFailure> {
        let addr = format!("{}:{}", endpoint.ip, endpoint.port);
        let mut tcpstream = TcpStream::connect(addr)?;
        let mut session = ClientSession::new(
            &self.tls_ctx,
            DNSNameRef::try_from_ascii_str("localhost").unwrap(),
        );
        let mut stream = Stream::new(&mut session, &mut tcpstream);
        send_vec(&mut stream, &request.as_bytes())?;
        debug!("Request sent.");
        let response = recv_vec(&mut stream)?;
        let response = std::str::from_utf8(&response)?;
        debug!("Response received: {}", response);
        let pattern = create_wildcard_regex(&expected_response)?;
        if pattern.is_match(&response) {
            Ok(())
        } else {
            println!("response: {}", response);
            println!("expected: {}", expected_response);
            Err(failure::err_msg("Mismatched response").into())
        }
    }
}

// Parameter `pattern` is the expected response pattern defined in `test.toml`.
// We invalidate all regex meta characters by escaping the pattern string.
// Then we empower character `*` (already escaped to `\\*`) with a special meaning as regex meta `.*`.
fn create_wildcard_regex(pattern: &str) -> Result<Regex, ExitFailure> {
    let escaped_pattern = regex::escape(&pattern.trim());
    let p = escaped_pattern.replace("\\*", ".*");
    Ok(Regex::new(&p)?)
}

fn get_tls_config() -> ClientConfig {
    let mut client_cfg = ClientConfig::new();

    client_cfg
        .dangerous()
        .set_certificate_verifier(Arc::new(NoServerAuth {}));
    client_cfg.versions.clear();
    client_cfg.versions.push(rustls::ProtocolVersion::TLSv1_2);

    client_cfg
}

struct NoServerAuth {}
impl rustls::ServerCertVerifier for NoServerAuth {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        _certs: &[rustls::Certificate],
        _hostname: webpki::DNSNameRef<'_>,
        _ocsp: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(rustls::ServerCertVerified::assertion())
    }
}
