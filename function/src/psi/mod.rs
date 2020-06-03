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

mod basic;
mod compute;
use anyhow::bail;
use compute::SetIntersection;
use std::format;
use std::io::{self, BufRead, BufReader, Write};
use std::prelude::v1::*;
use teaclave_types::{FunctionArguments, FunctionRuntime};

const IN_FILE1: &str = "in_file1";
const IN_FILE2: &str = "in_file2";
const OUT_FILE1: &str = "out_file1";
const OUT_FILE2: &str = "out_file2";

#[derive(Default)]
pub struct PSI;

impl PSI {
    pub const NAME: &'static str = "builtin_psi";

    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(
        &self,
        _arguments: FunctionArguments,
        runtime: FunctionRuntime,
    ) -> anyhow::Result<String> {
        let in_file1 = runtime.open_input(IN_FILE1)?;
        let in_file2 = runtime.open_input(IN_FILE2)?;

        let data1 = parse_input_data(in_file1)?;
        let data2 = parse_input_data(in_file2)?;

        let mut si = SetIntersection::new();
        if !si.psi_add_hash_data(data1, 0) {
            bail!("Invalid Input");
        }
        if !si.psi_add_hash_data(data2, 1) {
            bail!("Invalid Input");
        }
        si.compute();

        let mut output_file1 = runtime.create_output(OUT_FILE1)?;
        let mut output_file2 = runtime.create_output(OUT_FILE2)?;

        output_file1.write_all(&si.data[0].result)?;
        output_file2.write_all(&si.data[1].result)?;

        let summary = format!(
            "{}",
            si.data[0].result.len()
        );
        Ok(summary)
    }
}

fn parse_input_data(input: impl io::Read) -> anyhow::Result<Vec<u8>> {
    let mut samples: Vec<u8> = Vec::new();
    let mut reader = BufReader::new(input);
    let nums = reader.read_until(b'\n', &mut samples)?;
    if nums == 0 {
        bail!("Empty file");
    }
    Ok(samples)
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use serde_json::json;
    use teaclave_runtime::*;
    use teaclave_test_utils::*;
    use teaclave_types::*;

    pub fn run_tests() -> bool {
        run_tests!(test_psi)
    }

    fn test_psi() {
        let args = FunctionArguments::from_json(json!({
            "message": "Hello Teaclave!"
        }))
        .unwrap();

        let input_files = StagedFiles::default();
        let output_files = StagedFiles::default();

        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));
        let function = PSI;

        let summary = function.run(args, runtime).unwrap();
        assert_eq!(summary, "Hello Teaclave!");
    }
}
