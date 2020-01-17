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

use serde_derive::{Deserialize, Serialize};
use std::os::raw::c_int;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct StartServiceInput {
    pub fd: c_int,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct StartServiceOutput;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct InitEnclaveInput;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct InitEnclaveOutput;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct FinalizeEnclaveInput;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct FinalizeEnclaveOutput;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct RunTestInput;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct RunTestOutput {
    pub failed_count: usize,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ServeConnectionInput {
    pub socket_fd: c_int,
    pub port: u16,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct ServeConnectionOutput;

impl ServeConnectionInput {
    pub fn new(socket_fd: c_int, port: u16) -> ServeConnectionInput {
        ServeConnectionInput { socket_fd, port }
    }
}
