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

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use std::collections::HashMap;
use std::fmt::Write;

use crate::trait_defs::{WorkerHelper, WorkerInput};
use mesatee_core::{Error, ErrorKind, Result};

fn parse_input(data: Vec<u8>) -> Result<HashMap<String, u32>> {
    let data_list =
        String::from_utf8(data).map_err(|_| Error::from(ErrorKind::InvalidInputError))?;
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
pub(crate) fn private_join_and_compute(
    helper: &mut WorkerHelper,
    input: WorkerInput,
) -> Result<String> {
    // input identity: amount\n
    // output identity : sum_of_amount level_of_amount

    let number = input.input_files.len();

    let mut test_map: HashMap<String, (u32, usize)> = HashMap::new();
    for file_id in input.input_files.iter() {
        let plaintext = helper.read_file(file_id)?;
        let records = parse_input(plaintext)?;
        for (indentity, amount) in records.into_iter() {
            let value = test_map.get(&indentity).cloned().unwrap_or((0, 0));
            test_map.insert(indentity.to_owned(), (value.0 + amount, value.1 + 1));
        }
    }

    let mut output = String::new();
    for (identity, amount) in test_map.into_iter() {
        if amount.1 == number {
            writeln!(&mut output, "{} : {}", identity, amount.0)
                .map_err(|_| Error::from(ErrorKind::OutputGenerationError))?;
        }
    }
    let output_bytes = output.as_bytes().to_vec();

    for file_id in input.input_files.iter() {
        let _result_file = helper.save_file_for_file_owner(&output_bytes, file_id)?;
    }
    Ok("Finished".to_string())
}
