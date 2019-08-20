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
use serde_derive::Deserialize;
use std::collections::BTreeMap;
use structopt::StructOpt;

use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;

use mesatee_core;
use mesatee_sdk;

#[derive(Debug)]
enum EndpointType {
    TMS,
    TDFS,
    FNS,
}

/*fn open_connection<A: ToSocketAddrs>(
    addr: A,
    endpoint_type: EndpointType,
) -> Result<(), ExitFailure> {
    let enclave_identities = mesatee_sdk::sgx::load_and_verify_enclave_info(
        &enclave_info.enclave_info_file_path,
        &enclave_signers,
    );
    let channel = mesatee_core::sgx::channel::SgxTrustedChannel::new(addr, enclave_attrs)?;

    Ok(())
}*/

#[derive(Debug, StructOpt)]
/// MeasTEE client
struct Cli {
    #[structopt(
        short = "o",
        long,
        name = "FILE",
        parse(from_os_str)
    )]
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

    #[structopt(long, required = true)]
    /// Paths to MesaTEE auditor public keys
    auditor_keys: Vec<PathBuf>,

    #[structopt(long, required = true)]
    /// Paths to MesaTEE auditor endorsement signatures
    auditor_sigs: Vec<PathBuf>,

    #[structopt(long)]
    /// Path to Enclave info file
    enclave_info: PathBuf,
}

use mesatee_sdk::MesateeEnclaveInfo;

fn main() -> CliResult {
    let args = Cli::from_args();
    println!("{:?}", args);

    /*if args.auditor_keys.len() != args.auditor_sigs.len() {
        return Err(
            failure::err_msg("auditor_keys auditor_sigs have different sizes".to_string()).into(),
        );
    }*/

    //let mut auditors = vec![];

    //let enclave_info = MesateeEnclaveInfo::load(auditors, enclave_info_path)?;

    Ok(())
}
