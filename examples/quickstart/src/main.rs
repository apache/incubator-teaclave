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

use lazy_static::lazy_static;
use mesatee_sdk::{Error, ErrorKind, Mesatee, MesateeEnclaveInfo, Result, TaskStatus};
use std::net::SocketAddr;
use std::{thread, time};

lazy_static! {
    static ref TMS_ADDR: SocketAddr = "127.0.0.1:5554".parse().unwrap();
    static ref TDFS_ADDR: SocketAddr = "127.0.0.1:5065".parse().unwrap();
}
static PSI_TEST_DATA_1_PATH: &str = "./test_data/psi1.txt";
static PSI_TEST_DATA_2_PATH: &str = "./test_data/psi2.txt";

fn single_party_task(enclave_info: &MesateeEnclaveInfo) -> Result<()> {
    println!("[+] This is a single-party task: echo");

    let mesatee = Mesatee::new(enclave_info, "uid1", "token1", *TMS_ADDR, *TDFS_ADDR)?;
    let task = mesatee.create_task("echo")?;
    let ret = task.invoke_with_payload("haha")?;
    println!("{}", ret);

    Ok(())
}

fn multi_party_task(enclave_info: &MesateeEnclaveInfo) -> Result<()> {
    println!("[+] This is a multi-party task: psi");

    // Party 1 creates one PSI task
    let mesatee1 = Mesatee::new(enclave_info, "uid1", "token1", *TMS_ADDR, *TDFS_ADDR)?;
    let file1_id = mesatee1.upload_file(PSI_TEST_DATA_1_PATH)?;
    let task = mesatee1.create_task_with_collaborators("psi", &["uid2"], &[file1_id.as_str()])?;

    // Party 2 approves the task and invokes the task
    let task_id = task.task_id.clone();
    let mesatee2 = Mesatee::new(enclave_info, "uid2", "token2", *TMS_ADDR, *TDFS_ADDR)?;
    let file2_id = mesatee2.upload_file(PSI_TEST_DATA_2_PATH)?;
    mesatee2.approve_task_with_files(&task_id, &[file2_id.as_str()])?;

    let _ = task.invoke()?;

    // Party 1 waits for PSI results and get results from trusted FS
    let mut task_info = mesatee1
        .get_task(&task_id)?
        .task_info
        .ok_or_else(|| Error::from(ErrorKind::MissingValue))?;

    while task_info.status != TaskStatus::Finished {
        task_info = mesatee1
            .get_task(&task_id)?
            .task_info
            .ok_or_else(|| Error::from(ErrorKind::MissingValue))?;
        thread::sleep(time::Duration::from_secs(1));
    }

    let results = mesatee1.get_task_results(&task_id)?;
    println!("{:?}", results);
    let content = mesatee1.get_file(&results[0]);
    println!("{:?}", content);

    Ok(())
}

fn main() {
    // Load auditors' public keys and endorsement to TEE enclaves (digital signatures)
    let auditors = vec![
        (
            "../services/auditors/godzilla/godzilla.public.der",
            "../services/auditors/godzilla/godzilla.sign.sha256",
        ),
        (
            "../services/auditors/optimus_prime/optimus_prime.public.der",
            "../services/auditors/optimus_prime/optimus_prime.sign.sha256",
        ),
        (
            "../services/auditors/albus_dumbledore/albus_dumbledore.public.der",
            "../services/auditors/albus_dumbledore/albus_dumbledore.sign.sha256",
        ),
    ];
    let enclave_info_file_path = "../services/enclave_info.txt";

    let mesatee_enclave_info = MesateeEnclaveInfo::load(auditors, enclave_info_file_path).unwrap();

    match single_party_task(&mesatee_enclave_info) {
        Ok(_) => println!("[+] successfully invoke single-party task echo"),
        Err(e) => println!("[-] single-party task echo error: {}", e),
    }

    match multi_party_task(&mesatee_enclave_info) {
        Ok(_) => println!("[+] successfully invoke multi-party task psi"),
        Err(e) => println!("[-] multi-party task psi error: {}", e),
    }
}
