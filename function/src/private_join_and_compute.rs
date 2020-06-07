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

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use anyhow::{anyhow, bail, Result};
use std::convert::TryFrom;
use std::format;
use std::io::{self, BufRead, BufReader, Write};
use teaclave_types::{FunctionArguments, FunctionRuntime};

const IN_DATA: &str = "input_data";
const OUT_RESULT: &str = "output_data";

#[derive(Default)]
pub struct PrivateJoinAndCompute;

#[derive(serde::Deserialize)]
struct PrivateJoinAndComputeArguments {
    num_user: u32, // Number of users in the mutiple party computation
}

impl TryFrom<FunctionArguments> for PrivateJoinAndComputeArguments {
    type Error = anyhow::Error;

    fn try_from(arguments: FunctionArguments) -> Result<Self, Self::Error> {
        use anyhow::Context;
        serde_json::from_str(&arguments.into_string()).context("Cannot deserialize arguments")
    }
}

impl PrivateJoinAndCompute {
    pub const NAME: &'static str = "builtin-private-join-and-compute";

    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(
        &self,
        arguments: FunctionArguments,
        runtime: FunctionRuntime,
    ) -> anyhow::Result<String> {
        log::debug!("start traning...");
        let args = PrivateJoinAndComputeArguments::try_from(arguments)?;
        let num_user = args.num_user;
        if num_user < 2 {
            bail!("The demo requires at least two parties!");
        }
        let mut data1:Vec<u8> = Vec::new();
        for i in 0..num_user {
            let input_file_name = IN_DATA.to_string() + &i.to_string();            
            let mut input_io = runtime.open_input(&input_file_name[..])?;
            input_io.read_to_end(&mut data1)?;
        }

        for i in 0..num_user {
            let output_file_name = OUT_RESULT.to_string() + &i.to_string();     
            let mut output = runtime.create_output(&output_file_name[..])?;
            output.write_all(&data1.clone())?;
        }

        let summary = format!("{} parties joined.", num_user);
        Ok(summary)
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use serde_json::json;
    use std::untrusted::fs;
    use teaclave_crypto::*;
    use teaclave_runtime::*;
    use teaclave_test_utils::*;
    use teaclave_types::*;

    pub fn run_tests() -> bool {
        run_tests!(test_private_join_and_compute)
    }

    fn test_private_join_and_compute() {
        let t = "hello";
        assert_eq!(t, "hello");

    }
}
