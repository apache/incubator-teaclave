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

#[macro_use]
extern crate log;

mod common;
mod config;
mod mesapy;
mod multiparty_task;
mod psi;
mod sendrecv;
mod single_task;
mod test;
mod wasm;

use quicli::prelude::*;
use test::{Case, Runner};

fn init_logger() {
    let mut builder = env_logger::Builder::from_default_env();
    builder.filter_module("rustls", log::LevelFilter::Off);
    builder.init();
}

fn blackbox_test() {
    let test_data = read_file("test_data/test.toml").unwrap();
    let test = Case::new_from_toml(&test_data).unwrap();
    let runner = Runner::new();
    runner.run_test(&test).unwrap();
}

fn whitebox_test() {
    single_task::test_echo();
    single_task::test_bytes_plus_one();
    single_task::test_echo_file();
    single_task::test_file_bytes_plus_one();

    multiparty_task::test_concat_files();
    multiparty_task::test_swap_file();

    psi::test_psi();
    wasm::test_wasmi();
    mesapy::test_mesapy();
}

fn main() {
    init_logger();
    blackbox_test();
    whitebox_test();
    println!("[+] Done");
}
