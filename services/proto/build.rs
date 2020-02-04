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

use std::env;
use std::process::Command;
use std::str;

fn main() {
    let proto_files = [
        "src/proto/teaclave_authentication_service.proto",
        "src/proto/teaclave_common.proto",
        "src/proto/teaclave_database_service.proto",
        "src/proto/teaclave_execution_service.proto",
        "src/proto/teaclave_frontend_service.proto",
        "src/proto/teaclave_management_service.proto",
    ];

    let out_dir = env::var("OUT_DIR").expect("$OUT_DIR not set. Please build with cargo");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=proto_gen/templates/proto.j2");
    println!("cargo:rerun-if-changed=proto_gen/main.rs");

    for pf in proto_files.iter() {
        println!("cargo:rerun-if-changed={}", pf);
    }

    let c = Command::new("cargo")
        .args(&[
            "run",
            "--target-dir",
            "/tmp/proto_gen/target",
            "--manifest-path",
            "./proto_gen/Cargo.toml",
            "--",
            "-i",
            "src/proto",
            "-d",
            &out_dir,
            "-p",
        ])
        .args(&proto_files)
        .output()
        .expect("Generate proto failed");
    if !c.status.success() {
        panic!(
            "stdout: {:?}, stderr: {:?}",
            str::from_utf8(&c.stderr).unwrap(),
            str::from_utf8(&c.stderr).unwrap()
        );
    }
}
