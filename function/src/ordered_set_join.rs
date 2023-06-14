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

use anyhow::{anyhow, ensure};
use csv::{ReaderBuilder, StringRecord, Writer};
use std::cmp;
use std::convert::TryFrom;
use teaclave_types::{FunctionArguments, FunctionRuntime};

// Input data should be sorted by the specified index column.
const IN_DATA1: &str = "input_data1";
const IN_DATA2: &str = "input_data2";
// fusion output data
const OUT_RESULT: &str = "output_result";

#[derive(Default)]
pub struct OrderedSetJoin;

#[derive(serde::Deserialize)]
pub struct OrderedSetJoinArguments {
    // Start from 0.
    left_column: usize,
    right_column: usize,
    ascending: bool,
    // If it is set to true, drop the selected column of all files from the results.
    // If it is set to false, only keep the selected column of the first file.
    drop: bool,
}

impl TryFrom<FunctionArguments> for OrderedSetJoinArguments {
    type Error = anyhow::Error;

    fn try_from(arguments: FunctionArguments) -> Result<Self, Self::Error> {
        use anyhow::Context;
        serde_json::from_str(&arguments.into_string()).context("Cannot deserialize arguments")
    }
}

impl OrderedSetJoin {
    pub const NAME: &'static str = "builtin-ordered-set-join";

    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(
        &self,
        arguments: FunctionArguments,
        runtime: FunctionRuntime,
    ) -> anyhow::Result<String> {
        let args = OrderedSetJoinArguments::try_from(arguments)?;
        let mut rdr1 = ReaderBuilder::new()
            .has_headers(false)
            .from_reader(runtime.open_input(IN_DATA1)?);
        let mut rdr2 = ReaderBuilder::new()
            .has_headers(false)
            .from_reader(runtime.open_input(IN_DATA2)?);
        let mut wtr = Writer::from_writer(runtime.create_output(OUT_RESULT)?);

        let mut record1 = StringRecord::new();
        let mut record2 = StringRecord::new();
        let mut count = 0;
        ensure!(rdr1.read_record(&mut record1)?, "input1 is empty");
        ensure!(rdr2.read_record(&mut record2)?, "input2 is empty");

        loop {
            let fields1 = record1
                .get(args.left_column)
                .ok_or_else(|| anyhow!("invalid index"))?;
            let fields2 = record2
                .get(args.right_column)
                .ok_or_else(|| anyhow!("invalid index"))?;
            let order = &fields1.cmp(fields2);

            match order {
                cmp::Ordering::Equal => {
                    let new_record1 = if args.drop {
                        drop_column(&record1, args.left_column)
                    } else {
                        record1.clone()
                    };
                    let new_record2 = drop_column(&record2, args.right_column);
                    wtr.write_record(new_record1.iter().chain(&new_record2))?;
                    count += 1;
                    if !rdr1.read_record(&mut record1)? || !rdr2.read_record(&mut record2)? {
                        break;
                    }
                }
                cmp::Ordering::Less => {
                    if args.ascending {
                        if !rdr1.read_record(&mut record1)? {
                            break;
                        }
                    } else if !rdr2.read_record(&mut record2)? {
                        break;
                    }
                }
                cmp::Ordering::Greater => {
                    if args.ascending {
                        if !rdr2.read_record(&mut record2)? {
                            break;
                        }
                    } else if !rdr1.read_record(&mut record1)? {
                        break;
                    }
                }
            }
        }

        Ok(format!("{} records", count))
    }
}

pub fn drop_column(column: &StringRecord, index: usize) -> StringRecord {
    StringRecord::from(
        column
            .iter()
            .enumerate()
            .filter_map(|(i, e)| if i != index { Some(e) } else { None })
            .collect::<Vec<_>>(),
    )
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
        run_tests!(test_ordered_set_join)
    }

    fn test_ordered_set_join() {
        let arguments = FunctionArguments::from_json(json!({
            "left_column": 0,
            "right_column": 0,
            "ascending":true,
            "drop":true
        }))
        .unwrap();

        let base = Path::new("fixtures/functions/ordered_set_join");

        let user1_input = base.join("join0.csv");
        let output = base.join("output_join.csv");

        let user2_input = base.join("join1.csv");

        let input_files = StagedFiles::new(hashmap!(
            IN_DATA1 =>
            StagedFileInfo::new(&user1_input, TeaclaveFile128Key::random(), FileAuthTag::mock()),
            IN_DATA2 =>
            StagedFileInfo::new(&user2_input, TeaclaveFile128Key::random(), FileAuthTag::mock()),
        ));

        let output_files = StagedFiles::new(hashmap!(
            OUT_RESULT =>
            StagedFileInfo::new(&output, TeaclaveFile128Key::random(), FileAuthTag::mock()),
        ));

        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));
        let summary = OrderedSetJoin::new().run(arguments, runtime).unwrap();
        assert_eq!("120 records".to_string(), summary);

        let expected_output = "fixtures/functions/gbdt_training/train.txt";
        let result = fs::read_to_string(&output).unwrap();
        let expected = fs::read_to_string(expected_output).unwrap();
        assert_eq!(result.trim(), expected.trim());
    }
}
