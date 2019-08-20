// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use exitfailure::ExitFailure;
use failure::ResultExt;
use quicli::prelude::*;
use structopt::StructOpt;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::path::PathBuf;

use mesatee_core::config::OutboundDesc;
use mesatee_core::rpc::{self, sgx};
use tms_external_proto::{TaskRequest, TaskResponse};

type VerifiedEnclaveInfo = HashMap<String, (sgx::SgxMeasure, sgx::SgxMeasure)>;

#[derive(Debug, StructOpt)]
/// MeasTEE client
struct Cli {
    #[structopt(short = "o", long, name = "FILE", parse(from_os_str))]
    /// Write to FILE instead of stdout
    output: Option<PathBuf>,

    #[structopt(short = "v", long)]
    /// Make the operation more talkative
    verbose: bool,

    #[structopt(short = "V", long)]
    /// Show version number and quit
    version: bool,

    #[structopt(name = "SOCKET_ADDRESS", name = "IP_ADDRESS:PORT")]
    /// Address and port of the MeasTEE endpoint
    address: SocketAddr,

    #[structopt(short = "k", long, required = true)]
    /// SPACE seperated paths of MesaTEE auditor public keys
    auditor_keys: Vec<PathBuf>,

    #[structopt(short = "s", long, required = true)]
    /// SPACE seperated paths of MesaTEE auditor endorsement signatures.
    auditor_sigs: Vec<PathBuf>,

    #[structopt(short = "e", long)]
    /// Path to Enclave info file
    enclave_info: PathBuf,
}

fn main() -> CliResult {
    let args = Cli::from_args();
    if args.auditor_keys.len() != args.auditor_sigs.len() {
        return Err(
            failure::err_msg("auditor_keys auditor_sigs have different sizes".to_string()).into(),
        );
    }

    let mut keys = vec![];
    for key_path in args.auditor_keys.iter() {
        let mut buf = vec![];
        let mut f = File::open(key_path)?;
        let _ = f.read_to_end(&mut buf)?;
        keys.push(buf);
    }
    let mut enclave_signers = vec![];
    for auditor in keys.iter().zip(args.auditor_sigs.iter()) {
        let (key, sig_path) = auditor;
        enclave_signers.push((key.as_slice(), sig_path.as_path()));
    }
    let verified_enclave_info =
        sgx::load_and_verify_enclave_info(&args.enclave_info, enclave_signers.as_slice());

    // Initialization done.

    let tms_outbound_desc = OutboundDesc::new(*verified_enclave_info.get("tms").unwrap());
    let tms_desc = mesatee_core::config::TargetDesc::new(
        args.address.ip(),
        args.address.port(),
        tms_outbound_desc,
    );

    let message = r#"{"type":"Create","function_name":"fake","collaborator_list":[],"files":[],"user_id":"fake","user_token":"token"}"#;
    let request: TaskRequest = serde_json::from_str(&message)?;

    let mut channel = match &tms_desc.desc {
        OutboundDesc::Sgx(enclave_attrs) => rpc::channel::SgxTrustedChannel::<
            TaskRequest,
            TaskResponse,
        >::new(args.address, enclave_attrs.clone()),
    }?;

    let response = channel.invoke(request)?;
    println!("{:?}", response);

    Ok(())
}
