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

extern crate base64;
extern crate image;
#[cfg(feature = "mesalock_sgx")]
extern crate rustface;

use std::prelude::v1::*;

use std::convert::TryFrom;
use teaclave_types::{FunctionArguments, FunctionRuntime};

#[derive(Default)]
pub struct RustfaceDetector;

#[derive(serde::Deserialize)]
struct RustfaceArguments {
    image_base64: String,
}

impl TryFrom<FunctionArguments> for RustfaceArguments {
    type Error = anyhow::Error;

    fn try_from(arguments: FunctionArguments) -> Result<Self, Self::Error> {
        use anyhow::Context;
        serde_json::from_str(&arguments.into_string()).context("Cannot deserialize arguments")
    }
}

impl RustfaceDetector {
    pub const NAME: &'static str = "builtin-rustface-detector";

    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(
        &self,
        arguments: FunctionArguments,
        _runtime: FunctionRuntime,
    ) -> anyhow::Result<String> {
        let image_base64 = RustfaceArguments::try_from(arguments)?.image_base64;
        let vec = base64::decode(&image_base64).unwrap();
        let bytes: &[u8] = &vec;
        let img = image::load_from_memory(&bytes)?;

        let mut detector = rustface::create_default_detector()?;

        let faces = rustface::detect_faces(&mut *detector, img);
        let result = serde_json::to_string(&faces)?;
        log::debug!("{}", result);

        Ok(result)
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use serde_json::json;
    use teaclave_runtime::*;
    use teaclave_test_utils::*;
    use teaclave_types::*;

    pub fn run_tests() -> bool {
        true
    }
}
