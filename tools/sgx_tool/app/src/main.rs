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

use anyhow::anyhow;
use anyhow::Result;
use std::process;
use structopt::StructOpt;
use teaclave_binder::proto::{ECallCommand, RawJsonInput, RawJsonOutput};
use teaclave_binder::TeeBinder;
use teaclave_types::TeeServiceResult;

fn attestation(opt: &AttestationOpt) -> anyhow::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::new()
            .filter_or("TEACLAVE_LOG", "RUST_LOG")
            .write_style_or("TEACLAVE_LOG_STYLE", "RUST_LOG_STYLE"),
    );
    let tee = TeeBinder::new(env!("CARGO_PKG_NAME"))?;
    run(&tee, opt)?;
    tee.finalize();

    Ok(())
}

fn start_enclave_remote_attestation(tee: &TeeBinder, opt: &AttestationOpt) -> anyhow::Result<()> {
    let cmd = ECallCommand::Raw;
    let json = serde_json::to_string(opt)?;
    let input = RawJsonInput::new(json);
    match tee.invoke::<RawJsonInput, TeeServiceResult<RawJsonOutput>>(cmd, input) {
        Err(e) => Err(anyhow!("{:?}", e)),
        Ok(Err(e)) => Err(anyhow!("{:?}", e)),
        _ => Ok(()),
    }
}

fn run(tee: &TeeBinder, opt: &AttestationOpt) -> anyhow::Result<()> {
    start_enclave_remote_attestation(tee, opt)?;

    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(name = "teaclave_sgx_tool", about = "Teaclave SGX tool.")]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt, serde::Serialize)]
struct AttestationOpt {
    /// Attestation algorithm, supported algorithms are "sgx_epid" for IAS
    /// attestation and "sgx_ecdsa" for DCAP attestation.
    #[structopt(long, default_value = "sgx_epid")]
    algorithm: String,

    /// URL of attestation service.
    #[structopt(long, default_value = "https://api.trustedservices.intel.com:443")]
    url: String,

    /// API key for attestation service.
    #[structopt(long, default_value = "00000000000000000000000000000000")]
    key: String,

    /// SPID for attestation service.
    #[structopt(long, default_value = "00000000000000000000000000000000")]
    spid: String,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Dump current hardware and software information related with Intel SGX
    #[structopt(name = "status")]
    Status,
    /// Dump remote attestationation report
    #[structopt(name = "attestation")]
    Attestation(AttestationOpt),
}

fn status() {
    let cpuid = raw_cpuid::CpuId::new();
    println!(
        "Vendor: {}",
        cpuid
            .get_vendor_info()
            .as_ref()
            .map_or_else(|| "unknown", |vf| vf.as_string(),)
    );

    println!(
        "CPU Model: {}",
        cpuid.get_extended_function_info().as_ref().map_or_else(
            || "n/a",
            |extfuninfo| extfuninfo.processor_brand_string().unwrap_or("unreadable"),
        )
    );

    println!("SGX: ");

    println!(
        "  Has SGX: {}",
        cpuid
            .get_extended_feature_info()
            .as_ref()
            .map_or_else(|| "n/a".to_string(), |ext| ext.has_sgx().to_string(),)
    );

    let sgx_info = cpuid.get_sgx_info();
    match sgx_info {
        Some(sgx_info) => {
            println!("  Has SGX1: {}", sgx_info.has_sgx1());
            println!("  Has SGX2: {}", sgx_info.has_sgx2());
            println!("  Supports ENCLV instruction leaves EINCVIRTCHILD, EDECVIRTCHILD, and ESETCONTEXT: {}", sgx_info.has_enclv_leaves_einvirtchild_edecvirtchild_esetcontext());
            println!(
                "  Supports ENCLS instruction leaves ETRACKC, ERDINFO, ELDBC, and ELDUC: {}",
                sgx_info.has_encls_leaves_etrackc_erdinfo_eldbc_elduc()
            );
            println!(
                "  Bit vector of supported extended SGX features: {:#010X}",
                sgx_info.miscselect()
            );
            println!(
                "  Maximum supported enclave size in non-64-bit mode: 2^{}",
                sgx_info.max_enclave_size_non_64bit()
            );
            println!(
                "  Maximum supported enclave size in 64-bit mode: 2^{}",
                sgx_info.max_enclave_size_64bit()
            );
            println!("  Bits of SECS.ATTRIBUTES[127:0] set with ECREATE: {:#018X} (lower) {:#018X} (upper)", sgx_info.secs_attributes().0, sgx_info.secs_attributes().1);
            for i in sgx_info.iter() {
                match i {
                    raw_cpuid::SgxSectionInfo::Epc(epc) => {
                        println!("  EPC physical base: {:#018X}", epc.physical_base());
                        println!(
                            "  EPC size: {:#018X} ({}M)",
                            epc.size(),
                            epc.size() / 1024 / 1024
                        );
                    }
                }
            }
        }
        None => println!("  Intel SGX: n/a"),
    }

    println!(
        "  Supports flexible launch control: {}",
        cpuid
            .get_extended_feature_info()
            .as_ref()
            .map_or_else(|| "n/a".to_string(), |ext| ext.has_sgx_lc().to_string(),)
    );

    println!(
        "  SGX device: /dev/sgx {}, /dev/isgx {}",
        std::path::Path::new("/dev/sgx").exists(),
        std::path::Path::new("/dev/isgx").exists()
    );
    println!(
        "  AESM service: {}",
        std::path::Path::new("/var/run/aesmd/aesm.socket").exists()
    );

    for module in &["isgx", "sgx", "intel_sgx"] {
        println!("\nKernel module ({}):", module);
        if process::Command::new("modinfo")
            .arg(module)
            .status()
            .is_err()
        {
            println!("failed to execute modinfo {}", module);
        }
    }
}

fn main() -> Result<()> {
    let args = Opt::from_args();
    match args.command {
        Command::Status => status(),
        Command::Attestation(opt) => attestation(&opt)?,
    };
    Ok(())
}
