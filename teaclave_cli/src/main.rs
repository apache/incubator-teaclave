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
use structopt::StructOpt;

use std::fs;
use std::io::{self, Read, Write};
use std::{net, path};

use fns_proto::{InvokeTaskRequest, InvokeTaskResponse};
use mesatee_core::config::{OutboundDesc, TargetDesc};
use mesatee_core::rpc::channel;
use tdfs_external_proto::{DFSRequest, DFSResponse};
use teaclave_utils;
use teaclave_utils::EnclaveMeasurement;
use tms_external_proto::{TaskRequest, TaskResponse};

type EnclaveInfo = std::collections::HashMap<String, EnclaveMeasurement>;
type CliResult = Result<(), ExitFailure>;

#[derive(Debug, PartialEq)]
enum Endpoint {
    TMS,
    TDFS,
    FNS,
}

impl std::str::FromStr for Endpoint {
    type Err = String;

    fn from_str(s: &str) -> Result<Endpoint, String> {
        match s {
            "tms" => Ok(Endpoint::TMS),
            "tdfs" => Ok(Endpoint::TDFS),
            "fns" => Ok(Endpoint::FNS),
            _ => Err("Invalid endpoint specified".into()),
        }
    }
}

#[derive(Debug, StructOpt)]
struct AuditOpt {
    #[structopt(short = "k", long, required = true)]
    /// SPACE separated paths of Teaclave auditor public keys
    auditor_public_keys: Vec<path::PathBuf>,

    #[structopt(short = "s", long, required = true)]
    /// SPACE separated paths of Teaclave auditor endorsement signatures.
    auditor_signatures: Vec<path::PathBuf>,

    #[structopt(short = "c", long, required = true, name = "ENCLAVE_INFO_FILE")]
    /// Path to Enclave info file.
    enclave_info: path::PathBuf,
}

#[derive(Debug, StructOpt)]
struct ConnectOpt {
    #[structopt(short = "o", long, name = "INPUT_FILE", parse(from_os_str))]
    /// Write to FILE instead of stdout.
    output: Option<path::PathBuf>,

    #[structopt(short = "i", long, name = "OUTPUT_FILE", parse(from_os_str))]
    /// Read from FILE instead of stdin.
    input: Option<path::PathBuf>,

    #[structopt(short = "e", long, required = true)]
    /// Teaclave endpoint to connect to. Possible values are: tms, tdfs, fns.
    endpoint: Endpoint,

    #[structopt(name = "IP_ADDRESS:PORT", required = true)]
    /// Address and port of the Teaclave endpoint.
    addr: net::SocketAddr,

    #[structopt(short = "c", long, required = true, name = "ENCLAVE_INFO_FILE")]
    /// Path to Enclave info file.
    enclave_info: path::PathBuf,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Audit enclave info with auditors' public keys and signatures.
    #[structopt(name = "audit")]
    Audit(AuditOpt),
    /// Connect and send messages to Teaclave services
    #[structopt(name = "connect")]
    Connect(ConnectOpt),
}

#[derive(Debug, StructOpt)]
/// Teaclave command line tool
struct Cli {
    #[structopt(subcommand)]
    command: Command,
}

macro_rules! generate_runner_for {
    ($endpoint:ident, $request_type: tt, $response_type: tt) => {
        fn $endpoint<R: Read, W: Write>(
            enclave_info: &EnclaveInfo,
            addr: net::SocketAddr,
            reader: R,
            writer: W,
        ) -> Result<(), failure::Error> {
            let outbound_desc =
                OutboundDesc::new(*enclave_info.get(stringify!($endpoint)).unwrap());
            let target_desc = TargetDesc::new(addr, outbound_desc);

            let mut channel = match &target_desc.desc {
                OutboundDesc::Sgx(enclave_attrs) => channel::SgxTrustedChannel::<
                    $request_type,
                    $response_type,
                >::new(addr, enclave_attrs.clone()),
            }?;
            let request = serde_json::from_reader(reader)?;
            let response = channel.invoke(request)?;
            serde_json::to_writer(writer, &response)?;
            Ok(())
        }
    };
}

generate_runner_for!(tms, TaskRequest, TaskResponse);
generate_runner_for!(tdfs, DFSRequest, DFSResponse);
generate_runner_for!(fns, InvokeTaskRequest, InvokeTaskResponse);

fn connect(args: ConnectOpt) -> Result<(), failure::Error> {
    let enclave_info_content = fs::read_to_string(&args.enclave_info)?;
    let enclave_info = teaclave_utils::load_enclave_info(&enclave_info_content);

    let reader: Box<dyn Read> = match args.input {
        Some(i) => Box::new(io::BufReader::new(fs::File::open(i)?)),
        None => Box::new(io::stdin()),
    };
    let writer: Box<dyn Write> = match args.output {
        Some(o) => Box::new(io::BufWriter::new(fs::File::create(o)?)),
        None => Box::new(io::stdout()),
    };

    match args.endpoint {
        Endpoint::TMS => tms(&enclave_info, args.addr, reader, writer),
        Endpoint::TDFS => tdfs(&enclave_info, args.addr, reader, writer),
        Endpoint::FNS => fns(&enclave_info, args.addr, reader, writer),
    }
}

fn audit(args: AuditOpt) -> Result<(), failure::Error> {
    let enclave_info_content = fs::read_to_string(&args.enclave_info)?;
    let mut keys: Vec<Vec<u8>> = vec![];
    for key_path in args.auditor_public_keys.iter() {
        let buf = fs::read(key_path)?;
        keys.push(buf);
    }
    let mut signatures: Vec<Vec<u8>> = vec![];
    for path in &args.auditor_signatures {
        let signature = fs::read(path)?;
        signatures.push(signature);
    }

    if teaclave_utils::verify_enclave_info(enclave_info_content.as_bytes(), &keys, &signatures) {
        println!("Enclave info is successfully verified.");
        Ok(())
    } else {
        Err(failure::err_msg("Cannot verify the enclave info."))
    }
}

fn main() -> CliResult {
    let args = Cli::from_args();
    match args.command {
        Command::Audit(audit_args) => Ok(audit(audit_args)?),
        Command::Connect(connect_args) => Ok(connect(connect_args)?),
    }
}
