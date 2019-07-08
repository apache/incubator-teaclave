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

use crate::trait_defs::{WorkerHelper, WorkerInput};
use mesatee_core::{Error, ErrorKind, Result};

pub(crate) fn echo(_helper: &mut WorkerHelper, input: WorkerInput) -> Result<String> {
    match input.payload {
        Some(value) => Ok(value),
        None => Err(Error::from(ErrorKind::MissingValue)),
    }
}

pub(crate) fn bytes_plus_one(_helper: &mut WorkerHelper, input: WorkerInput) -> Result<String> {
    match input.payload {
        Some(value) => {
            let bytes: Vec<u8> = value.as_bytes().iter().map(|x| x + 1).collect();
            let result = String::from_utf8_lossy(&bytes).to_string();
            Ok(result)
        }
        None => Err(Error::from(ErrorKind::MissingValue)),
    }
}

pub(crate) fn echo_file(helper: &mut WorkerHelper, input: WorkerInput) -> Result<String> {
    match input.payload {
        Some(value) => {
            let bytes: Vec<u8> = helper.read_file(&value)?;
            let result = String::from_utf8_lossy(&bytes).to_string();
            Ok(result)
        }
        None => Err(Error::from(ErrorKind::MissingValue)),
    }
}

pub(crate) fn file_bytes_plus_one(helper: &mut WorkerHelper, input: WorkerInput) -> Result<String> {
    match input.payload {
        Some(value) => {
            let bytes: Vec<u8> = helper.read_file(&value)?;
            let modified_bytes: Vec<u8> = bytes.iter().map(|x| x + 1).collect();
            let result = String::from_utf8_lossy(&modified_bytes).to_string();
            Ok(result)
        }
        None => Err(Error::from(ErrorKind::MissingValue)),
    }
}

pub(crate) fn concat(helper: &mut WorkerHelper, input: WorkerInput) -> Result<String> {
    if input.input_files.len() < 2 {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }

    let mut result_bytes = Vec::<u8>::new();
    for file_id in input.input_files.iter() {
        let plaintext = helper.read_file(&file_id)?;
        result_bytes.extend_from_slice(&plaintext);
    }

    let result_string = String::from_utf8_lossy(&result_bytes).to_string();
    Ok(result_string)
}

pub(crate) fn swap_file(helper: &mut WorkerHelper, input: WorkerInput) -> Result<String> {
    if input.input_files.len() != 2 {
        return Err(Error::from(ErrorKind::InvalidInputError));
    }

    let file1 = &input.input_files[0];

    let file2 = &input.input_files[1];

    let plaintext1 = helper.read_file(&file1)?;
    let plaintext2 = helper.read_file(&file2)?;
    let mut result = Vec::<u8>::new();
    result.extend_from_slice(&plaintext1);
    result.extend_from_slice(&plaintext2);
    let result_file_id = helper.save_file_for_all_participants(&result)?;
    let _file_id = helper.save_file_for_file_owner(&plaintext2, file1)?;
    let _file_id = helper.save_file_for_file_owner(&plaintext1, file2)?;
    Ok(result_file_id)
}
