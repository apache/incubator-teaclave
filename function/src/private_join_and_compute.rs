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

use anyhow::{bail, Result};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::format;
use std::io::Write;
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;
use teaclave_types::{FunctionArguments, FunctionRuntime};

const IN_DATA: &str = "input_data";
const OUT_RESULT: &str = "output_data";

#[derive(Default)]
pub struct PrivateJoinAndCompute;

#[derive(serde::Deserialize)]
struct PrivateJoinAndComputeArguments {
    num_user: usize, // Number of users in the mutiple party computation
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
        let args = PrivateJoinAndComputeArguments::try_from(arguments)?;
        let num_user = args.num_user;
        if num_user < 2 {
            bail!("The demo requires at least two parties!");
        }

        let mut output = String::new();
        let data_0 = get_data(0, &runtime)?;
        let input_map_0 = parse_input(data_0)?;
        let mut res_map: HashMap<String, u32> = input_map_0;

        for i in 1..num_user {
            let data = get_data(i, &runtime)?;
            let input_map = parse_input(data)?;
            res_map = get_intersection_sum(&input_map, &res_map);
        }

        for (identity, amount) in res_map {
            fmt::write(&mut output, format_args!("{}, {}\n", identity, amount))?;
        }

        let output_bytes = output.as_bytes();

        for i in 0..num_user {
            let output_file_name = format!("{}{}", OUT_RESULT, i);
            let mut output = runtime.create_output(&output_file_name)?;
            output.write_all(&output_bytes)?;
        }

        let summary = format!("{} users join the task in total.", num_user);
        Ok(summary)
    }
}

fn get_data(user_id: usize, runtime: &FunctionRuntime) -> anyhow::Result<Vec<u8>> {
    let mut data: Vec<u8> = Vec::new();
    let input_file_name = format!("{}{}", IN_DATA, user_id);
    let mut input_io = runtime.open_input(&input_file_name)?;
    input_io.read_to_end(&mut data)?;
    Ok(data)
}

fn get_intersection_sum(
    map1: &HashMap<String, u32>,
    map2: &HashMap<String, u32>,
) -> HashMap<String, u32> {
    let mut res_map: HashMap<String, u32> = HashMap::new();
    for (identity, amount) in map1 {
        if map2.contains_key(identity) {
            let total = amount + map2[identity];
            res_map.insert(identity.to_owned(), total);
        }
    }
    res_map
}

fn parse_input(data: Vec<u8>) -> anyhow::Result<HashMap<String, u32>> {
    let data_list = String::from_utf8(data)?;
    let mut ret: HashMap<String, u32> = HashMap::new();
    for data_item in data_list.split('\n') {
        let pair = data_item.trim();
        if pair.len() < 3 {
            continue;
        }
        let kv_pair: Vec<&str> = pair.split(':').collect();
        if kv_pair.len() != 2 {
            continue;
        }
        let identity = kv_pair[0].trim().to_string();
        let amount = match kv_pair[1].trim().parse::<u32>() {
            Ok(amount) => amount,
            Err(_) => continue,
        };
        ret.insert(identity, amount);
    }
    Ok(ret)
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
        let arguments = FunctionArguments::from_json(json!({
            "num_user": 3
        }))
        .unwrap();

        let user0_input = "fixtures/functions/private_join_and_compute/three_party_data/bank_a.txt";
        let user0_output =
            "fixtures/functions/private_join_and_compute/three_party_results/user0_output.txt";

        let user1_input = "fixtures/functions/private_join_and_compute/three_party_data/bank_b.txt";
        let user1_output =
            "fixtures/functions/private_join_and_compute/three_party_results/user1_output.txt";

        let user2_input = "fixtures/functions/private_join_and_compute/three_party_data/bank_c.txt";
        let user2_output =
            "fixtures/functions/private_join_and_compute/three_party_results/user2_output.txt";

        let input_files = StagedFiles::new(hashmap!(
            "input_data0" =>
            StagedFileInfo::new(user0_input, TeaclaveFile128Key::random(), FileAuthTag::mock()),
            "input_data1" =>
            StagedFileInfo::new(user1_input, TeaclaveFile128Key::random(), FileAuthTag::mock()),
            "input_data2" =>
            StagedFileInfo::new(user2_input, TeaclaveFile128Key::random(), FileAuthTag::mock())
        ));

        let output_files = StagedFiles::new(hashmap!(
            "output_data0" =>
            StagedFileInfo::new(user0_output, TeaclaveFile128Key::random(), FileAuthTag::mock()),
            "output_data1" =>
            StagedFileInfo::new(user1_output, TeaclaveFile128Key::random(), FileAuthTag::mock()),
            "output_data2" =>
            StagedFileInfo::new(user2_output, TeaclaveFile128Key::random(), FileAuthTag::mock())
        ));

        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));

        let summary = PrivateJoinAndCompute::new()
            .run(arguments, runtime)
            .unwrap();

        let user0 = fs::read_to_string(&user0_output).unwrap();
        let user1 = fs::read_to_string(&user1_output).unwrap();
        let user2 = fs::read_to_string(&user2_output).unwrap();
        assert_eq!(&user0[..], summary);
        assert_eq!(&user1[..], summary);
        assert_eq!(&user2[..], summary);
    }
}
