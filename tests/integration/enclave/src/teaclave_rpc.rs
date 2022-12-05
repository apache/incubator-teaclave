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

use anyhow::Result;
use rustls::internal::pemfile;
use serde::{Deserialize, Serialize};
use std::io;
use std::untrusted::fs;
use teaclave_rpc::channel::*;
use teaclave_rpc::config::*;
use teaclave_rpc::endpoint::*;
use teaclave_rpc::server::*;
use teaclave_rpc::*;
use teaclave_types::TeaclaveServiceResponseError;
use teaclave_types::TeaclaveServiceResponseResult;

const END_FULLCHAIN: &str = "./fixtures/end_fullchain.pem";
const END_KEY: &str = "./fixtures/end_key.pem";

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "request", rename_all = "snake_case")]
enum EchoRequest {
    Say(SayRequest),
}

#[derive(Serialize, Deserialize, Debug)]
struct SayRequest {
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "response", rename_all = "snake_case")]
enum EchoResponse {
    Say(SayResponse),
}

#[derive(Serialize, Deserialize, Debug)]
struct SayResponse {
    message: String,
}

#[derive(Clone)]
struct EchoService;

impl TeaclaveService<EchoRequest, EchoResponse> for EchoService {
    fn handle_request(
        &self,
        request: teaclave_rpc::Request<EchoRequest>,
    ) -> TeaclaveServiceResponseResult<EchoResponse> {
        debug!("handle request: {:?}", request);
        let message = match request.message {
            EchoRequest::Say(s) => s.message,
        };
        Ok(EchoResponse::Say(SayResponse { message }))
    }
}

struct EchoClient {
    channel: SgxTrustedTlsChannel<EchoRequest, EchoResponse>,
}

impl EchoClient {
    fn new(channel: SgxTrustedTlsChannel<EchoRequest, EchoResponse>) -> Result<Self> {
        Ok(Self { channel })
    }

    fn say(&mut self, request: SayRequest) -> TeaclaveServiceResponseResult<SayResponse> {
        let request = EchoRequest::Say(request);
        let request = Request {
            metadata: std::collections::HashMap::<String, String>::new(),
            message: request,
        };
        let response = match self.channel.invoke(request) {
            Ok(response_result) => response_result,
            Err(_) => {
                return Err(TeaclaveServiceResponseError::InternalError(
                    "internal".to_string(),
                ));
            }
        };
        match response {
            EchoResponse::Say(r) => Ok(r),
        }
    }
}

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    start_echo_service();

    run_tests!(echo_success)
}

fn start_echo_service() {
    use std::thread;
    use std::time::Duration;
    thread::spawn(move || {
        let cert = pemfile::certs(&mut io::BufReader::new(
            fs::File::open(END_FULLCHAIN).unwrap(),
        ))
        .unwrap();
        let private_key =
            &pemfile::pkcs8_private_keys(&mut io::BufReader::new(fs::File::open(END_KEY).unwrap()))
                .unwrap()[0];
        let addr = "127.0.0.1:12345".parse().unwrap();
        let config = SgxTrustedTlsServerConfig::new()
            .server_cert(cert[0].as_ref(), &private_key.0)
            .unwrap();
        let mut server = SgxTrustedTlsServer::<EchoResponse, EchoRequest>::new(addr, config);
        server.start(EchoService).unwrap();
    });
    thread::sleep(Duration::from_secs(3));
}

fn echo_success() {
    let channel = Endpoint::new("localhost:12345").connect().unwrap();
    let mut client = EchoClient::new(channel).unwrap();
    let request = SayRequest {
        message: "Hello, World!".to_string(),
    };
    let response_result = client.say(request);
    debug!("{:?}", response_result);

    assert!(response_result.is_ok());
    assert!(response_result.unwrap().message == "Hello, World!");
}
