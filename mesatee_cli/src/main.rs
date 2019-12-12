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
use quicli::prelude::*;
use structopt::StructOpt;

use std::fs;
use std::io::{self, Read, Write};
use std::{net, path};

use fns_proto::{InvokeTaskRequest, InvokeTaskResponse};
use mesatee_core::config::{OutboundDesc, TargetDesc};
use mesatee_core::rpc::{channel, sgx};
use tdfs_external_proto::{DFSRequest, DFSResponse};
use tms_external_proto::{TaskRequest, TaskResponse};

type EnclaveInfo = std::collections::HashMap<String, (sgx::SgxMeasure, sgx::SgxMeasure)>;

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
/// MeasTEE client
struct Cli {
    #[structopt(short = "o", long, name = "IN_FILE", parse(from_os_str))]
    /// Write to FILE instead of stdout
    output: Option<path::PathBuf>,

    #[structopt(short = "i", long, name = "OUT_FILE", parse(from_os_str))]
    /// Read from FILE instead of stdin
    input: Option<path::PathBuf>,

    #[structopt(flatten)]
    verbosity: clap_flags::Verbosity,

    #[structopt(flatten)]
    logger: clap_flags::Log,

    #[structopt(short = "e", long, required = true)]
    /// MesaTEE endpoint to connect to. Possible values are: tms, tdfs, fns.
    endpoint: Endpoint,

    #[structopt(name = "SOCKET_ADDRESS", name = "IP_ADDRESS:PORT")]
    /// Address and port of the MeasTEE endpoint
    addr: net::SocketAddr,

    #[structopt(short = "k", long, required = true)]
    /// SPACE seperated paths of MesaTEE auditor public keys
    auditor_keys: Vec<path::PathBuf>,

    #[structopt(short = "s", long, required = true)]
    /// SPACE seperated paths of MesaTEE auditor endorsement signatures.
    auditor_sigs: Vec<path::PathBuf>,

    #[structopt(short = "c", long)]
    /// Path to Enclave info file
    enclave_info: path::PathBuf,
}

macro_rules! generate_runner_for {
    ($endpoint:ident, $request_type: tt, $response_type: tt) => {
        fn $endpoint<R: Read, W: Write>(
            enclave_info: &EnclaveInfo,
            addr: net::SocketAddr,
            reader: R,
            writer: W,
        ) -> Result<(), ExitFailure> {
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

fn main() -> CliResult {
    let args = Cli::from_args();
    if args.auditor_keys.len() != args.auditor_sigs.len() {
        return Err(
            failure::err_msg("auditor_keys auditor_sigs have different sizes".to_string()).into(),
        );
    }

    args.logger.log_all(args.verbosity.log_level())?;

    let mut keys = vec![];
    for key_path in args.auditor_keys.iter() {
        let mut buf = vec![];
        let mut f = fs::File::open(key_path)?;
        let _ = f.read_to_end(&mut buf)?;
        keys.push(buf);
    }
    let mut enclave_signers = vec![];
    for auditor in keys.iter().zip(args.auditor_sigs.iter()) {
        let (key, sig_path) = auditor;
        enclave_signers.push((key.as_slice(), sig_path.as_path()));
    }
    let enclave_info =
        sgx::load_and_verify_enclave_info(&args.enclave_info, enclave_signers.as_slice());

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
