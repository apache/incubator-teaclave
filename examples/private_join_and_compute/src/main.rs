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
use mesatee_sdk::{Mesatee, MesateeEnclaveInfo, TaskStatus};
use std::env;
use std::net::SocketAddr;

static FUNCTION_NAME: &str = "private_join_and_compute";

lazy_static! {
    static ref TMS_ADDR: SocketAddr = "127.0.0.1:5554".parse().unwrap();
    static ref TDFS_ADDR: SocketAddr = "127.0.0.1:5065".parse().unwrap();
}

fn print_usage() {
    let msg = "
    ./private_join_and_compute action action_parameters

    action and action_parameters :

    create_task your_user_id your_user_token your_file_path collaborator_user_id|collaborator_user_id2|...
    approve_task your_user_id your_user_token task_id expected_collaborator_id your_file_path
    launch_task your_user_id your_user_token task_id
    get_result your_user_id your_user_token task_id
    ";
    println!("usage: \n{}", msg);
}

fn create_task(
    info: &MesateeEnclaveInfo,
    user_id: &str,
    user_token: &str,
    file_path: &str,
    collaborator_user_id: &str,
) {
    let mesatee = Mesatee::new(info, user_id, user_token, *TMS_ADDR, *TDFS_ADDR).unwrap();
    let file_id = mesatee.upload_file(file_path).unwrap();
    let collaborator_list: Vec<&str> = collaborator_user_id.split('|').collect();
    let input_files: Vec<&str> = vec![&file_id];
    let task = mesatee
        .create_task_with_collaborators(FUNCTION_NAME, &collaborator_list, &input_files)
        .unwrap();

    println!("[+] {} created a {} task", user_id, FUNCTION_NAME);
    println!("[+] Encrypted data is provided");
    println!("[+] Collaborator user id is {}", collaborator_user_id);
    println!("[+] Task_id: {}", task.task_id);
    println!("[+] Please tell {} the task_id", collaborator_user_id);
}

fn approve_task(
    info: &MesateeEnclaveInfo,
    user_id: &str,
    user_token: &str,
    task_id: &str,
    expected_bank_id: &str,
    file_path: &str,
) {
    let mesatee = Mesatee::new(info, user_id, user_token, *TMS_ADDR, *TDFS_ADDR).unwrap();
    let file_id = mesatee.upload_file(file_path).unwrap();
    let task = mesatee.get_task(task_id).unwrap();
    let task_info = task.task_info.unwrap();
    assert_eq!(task.function_name.as_str(), FUNCTION_NAME);
    println!(
        "[+] {} verified that funciton name is {}",
        user_id, FUNCTION_NAME
    );
    assert_eq!(&task_info.creator, expected_bank_id);
    println!(
        "[+] {} verified that {} created the task",
        user_id, expected_bank_id
    );
    assert_eq!(task_info.status, TaskStatus::Created);
    println!("[+] {} verified that task status is Created", user_id);

    mesatee
        .approve_task_with_files(task_id, &[file_id.as_str()])
        .unwrap();
    println!(
        "[+] {} approved task {} and provided encrypted data",
        user_id, task_id
    );
}

fn launch_task(info: &MesateeEnclaveInfo, user_id: &str, user_token: &str, task_id: &str) {
    let mesatee = Mesatee::new(info, user_id, user_token, *TMS_ADDR, *TDFS_ADDR).unwrap();

    let task = mesatee.get_task(task_id).unwrap();
    let task_info = task.task_info.as_ref().unwrap();
    assert_eq!(task_info.status, TaskStatus::Ready);
    println!("[+] Task is ready");

    let _ = task.invoke().unwrap();
    println!("[+] {} launched task {}", &user_id, task_id);
    println!("[+] Task is finished");
}

fn get_result(info: &MesateeEnclaveInfo, user_id: &str, user_token: &str, task_id: &str) {
    let mesatee = Mesatee::new(info, user_id, user_token, *TMS_ADDR, *TDFS_ADDR).unwrap();

    let task = mesatee.get_task(task_id).unwrap();
    let task_info = task.task_info.unwrap();
    assert_eq!(task_info.status, TaskStatus::Finished);
    println!("[+] Task status is Finished");

    let result_files = mesatee.get_task_results(&task_id).unwrap();
    let content = mesatee.get_file(&result_files[0]).unwrap();
    let result = String::from_utf8(content).unwrap();
    println!("{} get result: \n{}", user_id, result);
}

fn main() {
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

    let args_string: Vec<String> = env::args().collect();
    let args: Vec<&str> = args_string.iter().map(|s| s.as_str()).collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let action = args[1];
    println!();
    match action {
        "create_task" => {
            if args.len() != 6 {
                print_usage();
                return;
            }
            create_task(&mesatee_enclave_info, args[2], args[3], args[4], args[5]);
        }
        "approve_task" => {
            if args.len() != 7 {
                print_usage();
                return;
            }
            approve_task(
                &mesatee_enclave_info,
                args[2],
                args[3],
                args[4],
                args[5],
                args[6],
            );
        }

        "launch_task" => {
            if args.len() != 5 {
                print_usage();
                return;
            }
            launch_task(&mesatee_enclave_info, args[2], args[3], args[4]);
        }

        "get_result" => {
            if args.len() != 5 {
                print_usage();
                return;
            }
            get_result(&mesatee_enclave_info, args[2], args[3], args[4]);
        }
        _ => {
            print_usage();
        }
    }
}
