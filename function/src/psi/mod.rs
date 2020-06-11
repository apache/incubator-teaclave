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

use std::env;
use std::fmt;
use std::format;
use std::io::{self, BufRead, BufReader, Read, Write};
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;
use teaclave_types::{FunctionArguments, FunctionRuntime};
mod basic;
mod compute;
use anyhow::{anyhow, bail, Result};
use compute::SetIntersection;

extern crate hex;

const IN_DATA1: &str = "input_data1";
const IN_DATA2: &str = "input_data2";
const OUT_RESULT1: &str = "output_result1";
const OUT_RESULT2: &str = "output_result2";

#[derive(Default)]
pub struct PrivateSetIntersection;

impl PrivateSetIntersection {
    pub const NAME: &'static str = "builtin-private-set-intersection";

    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(
        &self,
        _arguments: FunctionArguments,
        runtime: FunctionRuntime,
    ) -> anyhow::Result<String> {
        let input1 = runtime.open_input(IN_DATA1)?;
        let input2 = runtime.open_input(IN_DATA2)?;

        let data1 = parse_input_data(input1)?;
        let data2 = parse_input_data(input2)?;

        let mut si = SetIntersection::new();
        if !si.psi_add_hash_data(data1, 0) {
            bail!("Invalid Data");
        }
        if !si.psi_add_hash_data(data2, 1) {
            bail!("Invalid Data");
        }

        si.compute();

        let result1 = &si.data[0].result;
        let result2 = &si.data[1].result;

        let mut output1 = runtime.create_output(OUT_RESULT1)?;
        let mut output2 = runtime.create_output(OUT_RESULT2)?;

        for i in result1 {
            write!(&mut output1, "{}", i)?;
        }

        for i in result2 {
            write!(&mut output2, "{}", i)?;
        }
        Ok(format!("items in total"))
    }
}

fn parse_input_data(input: impl io::Read) -> anyhow::Result<Vec<u8>> {
    let mut samples: Vec<u8> = Vec::new();
    let reader = BufReader::new(input);
    for byte_result in reader.lines() {
        let byte = byte_result?;
        let result = hex::decode(byte)?;
        samples.extend_from_slice(&result);
    }
    Ok(samples)
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use serde_json::json;
    use std::untrusted::fs;
    use teaclave_crypto::*;
    use teaclave_runtime::*;
    use teaclave_test_utils::*;
    //use teaclave_types::*;

    pub fn run_tests() -> bool {
        true
    }
}
