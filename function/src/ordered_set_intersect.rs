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

use anyhow::bail;
use std::cmp;
use std::convert::TryFrom;
use std::format;
use std::io::{self, BufRead, BufReader, Write};
use teaclave_types::{FunctionArguments, FunctionRuntime};

extern crate hex;

// Input data should be a list of sorted hash values.

const IN_DATA1: &str = "input_data1";
const IN_DATA2: &str = "input_data2";
const OUT_RESULT1: &str = "output_result1";
const OUT_RESULT2: &str = "output_result2";

#[derive(Default)]
pub struct OrderedSetIntersect;

#[derive(serde::Deserialize)]
pub struct OrderedSetIntersectArguments {
    order: String,
}

impl TryFrom<FunctionArguments> for OrderedSetIntersectArguments {
    type Error = anyhow::Error;

    fn try_from(arguments: FunctionArguments) -> Result<Self, Self::Error> {
        use anyhow::Context;
        serde_json::from_str(&arguments.into_string()).context("Cannot deserialize arguments")
    }
}

impl OrderedSetIntersect {
    pub const NAME: &'static str = "builtin-ordered-set-intersect";

    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(
        &self,
        arguments: FunctionArguments,
        runtime: FunctionRuntime,
    ) -> anyhow::Result<String> {
        let input1 = runtime.open_input(IN_DATA1)?;
        let input2 = runtime.open_input(IN_DATA2)?;
        let mut output1 = runtime.create_output(OUT_RESULT1)?;
        let mut output2 = runtime.create_output(OUT_RESULT2)?;
        let args = OrderedSetIntersectArguments::try_from(arguments)?;
        let order = &args.order[..];
        let ascending_order = match order {
            "ascending" => true,
            "desending" => false,
            _ => bail!("Invalid order"),
        };

        let vec1 = parse_input_data(input1, ascending_order)?;
        let vec2 = parse_input_data(input2, ascending_order)?;
        let (result1, result2) = intersection_ordered_vec(&vec1, &vec2, ascending_order)?;

        let mut common_sets = 0;

        for item in result1 {
            write!(&mut output1, "{}", item as u32)?;
            if item {
                common_sets += 1;
            }
        }
        for item in result2 {
            write!(&mut output2, "{}", item as u32)?;
        }

        log::trace!("{}", common_sets);

        Ok(format!("{} common items", common_sets))
    }
}

fn parse_input_data(input: impl io::Read, ascending_order: bool) -> anyhow::Result<Vec<Vec<u8>>> {
    let mut samples: Vec<Vec<u8>> = Vec::new();
    let reader = BufReader::new(input);
    for (index, byte_result) in reader.lines().enumerate() {
        let byte = byte_result?;
        let result = hex::decode(byte)?;
        if index > 0 {
            // If vec has more than 2 elements, then verify the ordering
            let last_element = &samples[index - 1];
            if ascending_order && result < *last_element
                || !ascending_order && result > *last_element
            {
                bail!("Invalid ordering");
            }
        }
        samples.push(result)
    }

    Ok(samples)
}

fn intersection_ordered_vec(
    input1: &[Vec<u8>],
    input2: &[Vec<u8>],
    ascending_order: bool,
) -> anyhow::Result<(Vec<bool>, Vec<bool>)> {
    let v1_len = input1.len();
    let v2_len = input2.len();

    let mut res1 = std::vec![false; v1_len];
    let mut res2 = std::vec![false; v2_len];

    let mut i = 0;
    let mut j = 0;

    while i < v1_len && j < v2_len {
        let order = &input1[i].cmp(&input2[j]);
        match order {
            cmp::Ordering::Equal => {
                res1[i] = true;
                res2[j] = true;
                i += 1;
                j += 1;
            }
            cmp::Ordering::Less => {
                if ascending_order {
                    i += 1;
                } else {
                    j += 1;
                }
            }
            cmp::Ordering::Greater => {
                if ascending_order {
                    j += 1;
                } else {
                    i += 1;
                }
            }
        }
    }
    Ok((res1, res2))
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use serde_json::json;
    use std::path::Path;
    use std::untrusted::fs;
    use teaclave_crypto::*;
    use teaclave_runtime::*;
    use teaclave_test_utils::*;
    use teaclave_types::*;

    pub fn run_tests() -> bool {
        run_tests!(test_ordered_set_intersect)
    }

    fn test_ordered_set_intersect() {
        let arguments = FunctionArguments::from_json(json!({
            "order": "ascending"
        }))
        .unwrap();

        let base = Path::new("fixtures/functions/ordered_set_intersect");

        let user1_input = base.join("psi0.txt");
        let user1_output = base.join("output_psi0.txt");

        let user2_input = base.join("psi1.txt");
        let user2_output = base.join("output_psi1.txt");

        let input_files = StagedFiles::new(hashmap!(
            IN_DATA1 =>
            StagedFileInfo::new(&user1_input, TeaclaveFile128Key::random(), FileAuthTag::mock()),
            IN_DATA2 =>
            StagedFileInfo::new(&user2_input, TeaclaveFile128Key::random(), FileAuthTag::mock()),
        ));

        let output_files = StagedFiles::new(hashmap!(
            OUT_RESULT1 =>
            StagedFileInfo::new(&user1_output, TeaclaveFile128Key::random(), FileAuthTag::mock()),
            OUT_RESULT2 =>
            StagedFileInfo::new(&user2_output, TeaclaveFile128Key::random(), FileAuthTag::mock()),
        ));

        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));
        let summary = OrderedSetIntersect::new().run(arguments, runtime).unwrap();

        let user1_result = fs::read_to_string(&user1_output).unwrap();
        let user2_result = fs::read_to_string(&user2_output).unwrap();

        assert_eq!(&user1_result[..], "0101010");
        assert_eq!(&user2_result[..], "01101");
        assert_eq!(summary, "3 common items");
    }
}
