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

use anyhow;
use std::io;

use super::TeaclaveRuntime;
use teaclave_types::TeaclaveWorkerFileRegistry;
use teaclave_types::TeaclaveWorkerInputFileInfo;
use teaclave_types::TeaclaveWorkerOutputFileInfo;

pub struct DefaultRuntime {
    input_files: TeaclaveWorkerFileRegistry<TeaclaveWorkerInputFileInfo>,
    output_files: TeaclaveWorkerFileRegistry<TeaclaveWorkerOutputFileInfo>,
}

impl DefaultRuntime {
    pub fn new(
        input_files: TeaclaveWorkerFileRegistry<TeaclaveWorkerInputFileInfo>,
        output_files: TeaclaveWorkerFileRegistry<TeaclaveWorkerOutputFileInfo>,
    ) -> DefaultRuntime {
        DefaultRuntime {
            input_files,
            output_files,
        }
    }
}

impl TeaclaveRuntime for DefaultRuntime {
    fn open_input(&self, identifier: &str) -> anyhow::Result<Box<dyn io::Read>> {
        let file_info = self
            .input_files
            .entries
            .get(identifier)
            .ok_or_else(|| anyhow::anyhow!("Invalid input file identifier."))?;

        let readable = file_info.get_readable_io()?;
        Ok(readable)
    }

    fn create_output(&self, identifier: &str) -> anyhow::Result<Box<dyn io::Write>> {
        let file_info = self
            .output_files
            .entries
            .get(identifier)
            .ok_or_else(|| anyhow::anyhow!("Invalide output file identifier"))?;

        let writable = file_info.get_writable_io()?;
        Ok(writable)
    }
}
