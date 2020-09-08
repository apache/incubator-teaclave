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

#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
#[macro_use]
extern crate sgx_tstd as std;

use std::prelude::v1::*;

use serde_json::Value;
use teaclave_attestation::report::SgxQuote;
use teaclave_attestation::EndorsedAttestationReport;
use teaclave_attestation::{key, AttestationConfig};
use teaclave_binder::proto::{
    ECallCommand, FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    RawJsonInput, RawJsonOutput,
};
use teaclave_binder::{handle_ecall, register_ecall_handler};
use teaclave_service_enclave_utils::ServiceEnclave;
use teaclave_types::TeeServiceError;
use teaclave_types::{self, TeeServiceResult};

fn attestation(raw_json_input: &RawJsonInput) -> anyhow::Result<()> {
    let v: serde_json::Value = serde_json::from_str(&raw_json_input.json)?;
    let attestation_config = AttestationConfig::new(
        v["algorithm"].as_str().unwrap(),
        v["url"].as_str().unwrap(),
        v["key"].as_str().unwrap(),
        v["spid"].as_str().unwrap(),
    )?;
    let key_pair = key::NistP256KeyPair::new()?;
    let report = match *attestation_config {
        AttestationConfig::NoAttestation => EndorsedAttestationReport::default(),
        AttestationConfig::WithAttestation(ref config) => {
            EndorsedAttestationReport::new(config, key_pair.pub_k())?
        }
    };
    let attn_report: Value = serde_json::from_slice(&report.report)?;
    let sgx_quote_body = {
        let quote_encoded = attn_report["isvEnclaveQuoteBody"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("report error"))?;
        let quote_raw = base64::decode(&quote_encoded.as_bytes())?;
        SgxQuote::parse_from(quote_raw.as_slice())?
    };
    println!("Remote Attestation Report:");
    println!("{}", serde_json::to_string_pretty(&attn_report)?);
    println!();
    println!("ISV Enclave Quote Body:");
    println!("{:?}", sgx_quote_body);
    Ok(())
}

#[handle_ecall]
fn handle_remote_attestation(input: &RawJsonInput) -> TeeServiceResult<RawJsonOutput> {
    match attestation(input) {
        Ok(_) => Ok(RawJsonOutput::default()),
        Err(e) => {
            log::error!("Failed to start the service: {}", e);
            Err(TeeServiceError::ServiceError)
        }
    }
}

#[handle_ecall]
fn handle_init_enclave(_: &InitEnclaveInput) -> TeeServiceResult<InitEnclaveOutput> {
    ServiceEnclave::init(env!("CARGO_PKG_NAME"))?;
    Ok(InitEnclaveOutput)
}

#[handle_ecall]
fn handle_finalize_enclave(_: &FinalizeEnclaveInput) -> TeeServiceResult<FinalizeEnclaveOutput> {
    ServiceEnclave::finalize()?;
    Ok(FinalizeEnclaveOutput)
}

register_ecall_handler!(
    type ECallCommand,
    (ECallCommand::Raw, RawJsonInput, RawJsonOutput),
    (ECallCommand::InitEnclave, InitEnclaveInput, InitEnclaveOutput),
    (ECallCommand::FinalizeEnclave, FinalizeEnclaveInput, FinalizeEnclaveOutput),
);
