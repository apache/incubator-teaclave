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

use std::io::prelude::*;

use std::collections::HashSet;
use std::convert::TryFrom;
use std::io::BufReader;
use teaclave_types::{FunctionArguments, FunctionRuntime};

#[derive(Default)]
pub struct PasswordCheck;

#[derive(serde::Deserialize)]
struct PasswordCheckArguments;

impl TryFrom<FunctionArguments> for PasswordCheckArguments {
    type Error = anyhow::Error;

    fn try_from(arguments: FunctionArguments) -> Result<Self, Self::Error> {
        use anyhow::Context;
        serde_json::from_str(&arguments.into_string()).context("Cannot deserialize arguments")
    }
}

impl PasswordCheck {
    pub const NAME: &'static str = "builtin-password-check";

    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(&self, _: FunctionArguments, runtime: FunctionRuntime) -> anyhow::Result<String> {
        let password_file = runtime.open_input("password")?;
        let password = BufReader::new(password_file)
            .lines()
            .next()
            .unwrap()
            .unwrap()
            .trim()
            .to_owned();
        let exposed_passwords_file = runtime.open_input("exposed_passwords")?;
        let exposed_passwords: HashSet<String> = BufReader::new(exposed_passwords_file)
            .lines()
            .map(|l| l.expect("Could not parse line").trim().to_string())
            .collect::<Vec<String>>()
            .iter()
            .cloned()
            .collect();
        if exposed_passwords.contains(&password) {
            Ok("true".to_string())
        } else {
            Ok("false".to_string())
        }
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use std::path::Path;
    use teaclave_crypto::*;
    use teaclave_runtime::*;
    use teaclave_test_utils::*;
    use teaclave_types::*;

    pub fn run_tests() -> bool {
        run_tests!(test_password_check)
    }

    fn test_password_check() {
        let password_input = Path::new("fixtures/functions/password_check/password.txt");
        let exposed_passwords_input =
            Path::new("fixtures/functions/password_check/exposed_passwords.txt");
        let arguments = FunctionArguments::default();

        let input_files = StagedFiles::new(hashmap!(
            "password" => StagedFileInfo::new(password_input, TeaclaveFile128Key::random(), FileAuthTag::mock()),
            "exposed_passwords" => StagedFileInfo::new(exposed_passwords_input, TeaclaveFile128Key::random(), FileAuthTag::mock()),
        ));
        let output_files = StagedFiles::new(hashmap!());
        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));

        let result = PasswordCheck::new().run(arguments, runtime).unwrap();

        assert_eq!(result, "true");
    }
}
