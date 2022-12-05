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

use serde::{Deserialize, Serialize};
use std::convert::From;

pub enum ECallCommand {
    StartService,
    InitEnclave,
    FinalizeEnclave,
    RunTest,
    Raw,
    Unimplemented,
}

impl From<u32> for ECallCommand {
    #[inline]
    fn from(cmd: u32) -> ECallCommand {
        match cmd {
            0x0000_1000 => ECallCommand::StartService,
            0x0000_1001 => ECallCommand::InitEnclave,
            0x0000_1002 => ECallCommand::FinalizeEnclave,
            0x0000_1003 => ECallCommand::RunTest,
            0x0000_1004 => ECallCommand::Raw,
            _ => ECallCommand::Unimplemented,
        }
    }
}

impl From<ECallCommand> for u32 {
    #[inline]
    fn from(cmd: ECallCommand) -> u32 {
        match cmd {
            ECallCommand::StartService => 0x0000_1000,
            ECallCommand::InitEnclave => 0x0000_1001,
            ECallCommand::FinalizeEnclave => 0x0000_1002,
            ECallCommand::RunTest => 0x0000_1003,
            ECallCommand::Raw => 0x0000_1004,
            ECallCommand::Unimplemented => 0xffff_ffff,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StartServiceInput {
    pub config: teaclave_config::RuntimeConfig,
}

impl StartServiceInput {
    pub fn new(config: teaclave_config::RuntimeConfig) -> Self {
        Self { config }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StartServiceOutput;

#[derive(Serialize, Deserialize, Debug)]
pub struct InitEnclaveInput;

#[derive(Serialize, Deserialize, Debug)]
pub struct InitEnclaveOutput;

#[derive(Serialize, Deserialize, Debug)]
pub struct FinalizeEnclaveInput;

#[derive(Serialize, Deserialize, Debug)]
pub struct FinalizeEnclaveOutput;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct RunTestInput {
    pub test_names: Vec<String>,
}

impl RunTestInput {
    pub fn new(test_names: Vec<String>) -> Self {
        Self { test_names }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RunTestOutput;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct RawJsonInput {
    pub json: String,
}

impl RawJsonInput {
    pub fn new(json: impl ToString) -> Self {
        Self {
            json: json.to_string(),
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct RawJsonOutput {
    pub json: String,
}
