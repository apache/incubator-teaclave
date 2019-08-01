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

use crate::worker::{FunctionType, Worker, WorkerContext};
use mesatee_core::{Error, ErrorKind, Result};

pub struct PrivateJoinAndComputeWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<PrivateJoinAndComputeWorkerInput>,
}
struct PrivateJoinAndComputeWorkerInput {
    file_list: Vec<String>,
}
impl PrivateJoinAndComputeWorker {
    pub fn new() -> Self {
        PrivateJoinAndComputeWorker {
            worker_id: 0,
            func_name: "private_join_and_compute".to_string(),
            func_type: FunctionType::Multiparty,
            input: None,
        }
    }
}
impl Worker for PrivateJoinAndComputeWorker {
    fn function_name(&self) -> &str {
        self.func_name.as_str()
    }
    fn function_type(&self) -> FunctionType {
        self.func_type
    }
    fn set_id(&mut self, worker_id: u32) {
        self.worker_id = worker_id;
    }
    fn id(&self) -> u32 {
        self.worker_id
    }
    fn prepare_input(
        &mut self,
        _dynamic_input: Option<String>,
        file_ids: Vec<String>,
    ) -> Result<()> {
        if file_ids.len() < 2 {
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
        self.input = Some(PrivateJoinAndComputeWorkerInput {
            file_list: file_ids,
        });
        Ok(())
    }

    fn execute(&mut self, context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        let mut counter_map: HashMap<String, usize> = HashMap::new();
        let mut add_map: HashMap<String, u32> = HashMap::new();

        let number = input.file_list.len();

        for file_id in input.file_list.iter() {
            let plaintext = context.read_file(file_id)?;
            let records = parse_input(plaintext)?;
            for (indentity, amount) in records.into_iter() {
                let value = counter_map.get(&indentity).cloned().unwrap_or(0);
                counter_map.insert(indentity.to_owned(), value + 1);
                let value = add_map.get(&indentity).cloned().unwrap_or(0);
                add_map.insert(indentity, value + amount);
            }
        }

        // get intersection set;
        counter_map.retain(|_, &mut v| v == number);

        let mut output = String::new();

        for (identity, amount) in add_map.into_iter() {
            if counter_map.contains_key(&identity) {
                writeln!(&mut output, "{} : {}", identity, amount)
                    .map_err(|_| Error::from(ErrorKind::OutputGenerationError))?;
            }
        }

        let output_bytes = output.as_bytes().to_vec();

        for file_id in input.file_list.iter() {
            let _result_file = context.save_file_for_file_owner(&output_bytes, file_id)?;
        }
        Ok("Finished".to_string())
    }
}

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
