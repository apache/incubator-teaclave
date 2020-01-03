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

use lazy_static::lazy_static;
use mesatee_sdk::{Mesatee, MesateeEnclaveInfo, TaskStatus};
use std::net::SocketAddr;
use std::path;
use std::path::PathBuf;
use std::{thread, time};
use structopt::StructOpt;

lazy_static! {
    static ref TMS_ADDR: SocketAddr = "127.0.0.1:5554".parse().unwrap();
    static ref TDFS_ADDR: SocketAddr = "127.0.0.1:5065".parse().unwrap();
}

#[derive(Debug, StructOpt)]
struct EchoOpt {
    #[structopt(short = "e", required = true)]
    enclave_info: PathBuf,

    #[structopt(short = "m", required = true)]
    message: String,
}

#[derive(Debug, StructOpt)]
struct PsiOpt {
    #[structopt(short = "e", required = true)]
    enclave_info: PathBuf,

    #[structopt(short = "f", required = true)]
    files: Vec<path::PathBuf>,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Echo
    #[structopt(name = "echo")]
    Echo(EchoOpt),

    /// Private set intersection
    #[structopt(name = "psi")]
    Psi(PsiOpt),
}

#[derive(Debug, StructOpt)]
/// Quickstart example.
struct Cli {
    #[structopt(subcommand)]
    command: Command,
}

fn echo(args: EchoOpt) {
    println!("[+] Invoke echo function");
    let auditors = vec![]; // legacy

    let enclave_info =
        MesateeEnclaveInfo::load(auditors, args.enclave_info.to_str().unwrap()).expect("load");

    let mesatee =
        Mesatee::new(&enclave_info, "uid1", "token1", *TMS_ADDR, *TDFS_ADDR).expect("new");
    let task = mesatee.create_task("echo").expect("create");
    let response = task.invoke_with_payload(&args.message).expect("invoke");
    println!("{}", response);
}

fn psi(args: PsiOpt) {
    println!("[+] Invoke psi function");
    let auditors = vec![]; // legacy

    let enclave_info =
        MesateeEnclaveInfo::load(auditors, args.enclave_info.to_str().unwrap()).expect("load");

    // Party 1 creates one PSI task
    let mesatee1 =
        Mesatee::new(&enclave_info, "uid1", "token1", *TMS_ADDR, *TDFS_ADDR).expect("new");
    let file1_id = mesatee1
        .upload_file(args.files[0].to_str().unwrap())
        .expect("upload_file");
    let task = mesatee1
        .create_task_with_collaborators("psi", &["uid2"], &[file1_id.as_str()])
        .expect("create_task");

    // Party 2 approves the task and invokes the task
    let task_id = task.task_id.clone();
    let mesatee2 =
        Mesatee::new(&enclave_info, "uid2", "token2", *TMS_ADDR, *TDFS_ADDR).expect("new");
    let file2_id = mesatee2
        .upload_file(args.files[1].to_str().unwrap())
        .expect("upload_file");
    mesatee2
        .approve_task_with_files(&task_id, &[file2_id.as_str()])
        .expect("aprrove_task_with_files");

    let _ = task.invoke().expect("invoke");

    // Party 1 waits for PSI results and get results from trusted FS
    let mut task_info = mesatee1
        .get_task(&task_id)
        .expect("get_task")
        .task_info
        .expect("task_info");

    while task_info.status != TaskStatus::Finished {
        task_info = mesatee1
            .get_task(&task_id)
            .expect("get_task")
            .task_info
            .expect("task_info");
        thread::sleep(time::Duration::from_secs(1));
    }

    let results = mesatee1
        .get_task_results(&task_id)
        .expect("get_task_results");
    println!("{:?}", results);
    let content = mesatee1.get_file(&results[0]).expect("get_file");
    println!("{:?}", content);
}

fn main() {
    let args = Cli::from_args();
    match args.command {
        Command::Echo(echo_args) => echo(echo_args),
        Command::Psi(psi_args) => psi(psi_args),
    }
}
