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

use crate::ocall::handle_file_request;

use std::collections::HashMap;
use std::prelude::v1::*;
use std::sync::{Arc, SgxMutex as Mutex};

use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::{ExecutionResult, StagedFunction, StagedTask, TaskStatus};
use teaclave_worker::Worker;

use anyhow::Result;
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct TeaclaveExecutionService {
    worker: Arc<Worker>,
    scheduler_client: Arc<Mutex<TeaclaveSchedulerClient>>,
}

impl TeaclaveExecutionService {
    pub(crate) fn new(scheduler_service_endpoint: Endpoint) -> Result<Self> {
        let mut i = 0;
        let channel = loop {
            match scheduler_service_endpoint.connect() {
                Ok(channel) => break channel,
                Err(_) => {
                    anyhow::ensure!(i < 3, "failed to connect to storage service");
                    log::debug!("Failed to connect to storage service, retry {}", i);
                    i += 1;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        };
        let scheduler_client = Arc::new(Mutex::new(TeaclaveSchedulerClient::new(channel)?));
        Ok(TeaclaveExecutionService {
            worker: Arc::new(Worker::default()),
            scheduler_client,
        })
    }

    pub(crate) fn start(&mut self) -> Result<()> {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let scheduler_client = self.scheduler_client.clone();
            let mut client = match scheduler_client.lock() {
                Ok(client) => client,
                Err(e) => {
                    log::error!("Start Error: {:?}", e);
                    continue;
                }
            };

            let request = PullTaskRequest {};
            log::debug!("pull_task");
            let response = match client.pull_task(request) {
                Ok(response) => response,
                Err(e) => {
                    log::error!("PullTask Error: {:?}", e);
                    continue;
                }
            };
            drop(client); // drop mutex guard

            log::debug!("response: {:?}", response);
            let staged_task = response.staged_task;
            let result = self.invoke_task(&staged_task).unwrap();
            log::debug!("result: {:?}", result);
            match self.update_task_result(&staged_task.task_id, result) {
                Ok(_) => (),
                Err(e) => {
                    log::error!("UpdateResult Error: {:?}", e);
                    continue;
                }
            }
            match self.update_task_status(&staged_task.task_id, TaskStatus::Finished) {
                Ok(_) => (),
                Err(e) => {
                    log::error!("UpdateTask Error: {:?}", e);
                    continue;
                }
            }
        }
    }

    fn invoke_task(&mut self, task: &StagedTask) -> Result<ExecutionResult> {
        log::debug!("invoke_task");
        self.update_task_status(&task.task_id, TaskStatus::Running)?;
        let invocation = prepare_task(&task);
        let worker = Worker::default();
        let summary = worker.invoke_function(invocation)?;
        log::debug!("summary: {:?}", summary);
        finalize_task(&task)?;
        let mut result = ExecutionResult::default();
        result.return_value = summary.as_bytes().to_vec();

        Ok(result)
    }

    fn update_task_result(&mut self, task_id: &Uuid, result: ExecutionResult) -> Result<()> {
        log::debug!("update_task_result");
        let request = UpdateTaskResultRequest::new(
            task_id.to_owned(),
            &result.return_value,
            result.output_file_hash,
        );
        let _response = self
            .scheduler_client
            .clone()
            .lock()
            .map_err(|_| anyhow::anyhow!("Cannot lock scheduler client"))?
            .update_task_result(request)?;

        Ok(())
    }

    fn update_task_status(&mut self, task_id: &Uuid, task_status: TaskStatus) -> Result<()> {
        log::debug!("update_task_status");
        let request = UpdateTaskStatusRequest::new(task_id.to_owned(), task_status);
        let _response = self
            .scheduler_client
            .clone()
            .lock()
            .map_err(|_| anyhow::anyhow!("Cannot lock scheduler client"))?
            .update_task_status(request)?;

        Ok(())
    }
}

fn finalize_task(task: &StagedTask) -> Result<()> {
    use std::path::Path;
    use teaclave_types::*;

    let agent_dir = format!("/tmp/teaclave_agent/{}", task.task_id);
    let agent_dir_path = Path::new(&agent_dir);

    let mut file_request_info = vec![];
    for (key, value) in task.output_data.iter() {
        let mut src = agent_dir_path.to_path_buf();
        src.push(&format!("{}.out", key));
        let handle_file_info = HandleFileInfo::new(&src, &value.url);
        file_request_info.push(handle_file_info);
    }
    let request = FileAgentRequest::new(HandleFileCommand::Upload, file_request_info);
    handle_file_request(request)?;

    Ok(())
}

fn prepare_task(task: &StagedTask) -> StagedFunction {
    use std::path::Path;
    use std::path::PathBuf;
    use std::untrusted::fs;
    use std::untrusted::path::PathEx;
    use teaclave_types::*;

    let runtime_name = "default".to_string();
    let executor_type = task.executor_type();
    let function_name = task.function_name.clone();
    let function_payload = String::from_utf8_lossy(&task.function_payload).to_string();
    let function_arguments = task.function_arguments.clone();

    let agent_dir = format!("/tmp/teaclave_agent/{}", task.task_id);
    let agent_dir_path = Path::new(&agent_dir);
    if !agent_dir_path.exists() {
        fs::create_dir_all(agent_dir_path).unwrap();
    }

    let mut input_file_map: HashMap<String, (PathBuf, FileCrypto)> = HashMap::new();
    let mut file_request_info = vec![];
    for (key, value) in task.input_data.iter() {
        let mut dest = agent_dir_path.to_path_buf();
        dest.push(&format!("{}.in", key));
        let info = HandleFileInfo::new(&dest, &value.url);
        file_request_info.push(info);
        input_file_map.insert(key.to_string(), (dest, value.crypto_info));
    }
    let request = FileAgentRequest::new(HandleFileCommand::Download, file_request_info);
    handle_file_request(request).unwrap();

    let mut converted_input_file_map: HashMap<String, StagedFileInfo> = HashMap::new();
    for (key, value) in input_file_map.iter() {
        let (from, crypto_info) = value;
        let mut dest = from.clone();
        let mut file_name = dest.file_name().unwrap().to_os_string();
        file_name.push(".converted");
        dest.set_file_name(file_name);
        let input_file_info = convert_encrypted_input_file(from, *crypto_info, &dest).unwrap();
        converted_input_file_map.insert(key.to_string(), input_file_info);
    }
    let input_files = StagedFiles::new(converted_input_file_map);

    let mut output_file_map: HashMap<String, StagedFileInfo> = HashMap::new();
    for (key, value) in task.output_data.iter() {
        let mut dest = agent_dir_path.to_path_buf();
        dest.push(&format!("{}.out", key));
        let crypto = match value.crypto_info {
            FileCrypto::TeaclaveFile128(crypto) => crypto,
            _ => unimplemented!(),
        };
        let output_info = StagedFileInfo::new(&dest, crypto);
        output_file_map.insert(key.to_string(), output_info);
    }
    let output_files = StagedFiles::new(output_file_map);

    StagedFunction::new()
        .name(function_name)
        .payload(function_payload)
        .arguments(function_arguments)
        .input_files(input_files)
        .output_files(output_files)
        .runtime_name(runtime_name)
        .executor_type(executor_type)
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use std::format;
    use teaclave_types::*;
    use url::Url;
    use uuid::Uuid;

    pub fn test_invoke_echo() {
        let task_id = Uuid::new_v4();
        let function_arguments = FunctionArguments::new(hashmap!(
            "message" => "Hello, Teaclave!"
        ));
        let staged_task = StagedTask::new()
            .task_id(task_id)
            .function_name("echo")
            .function_arguments(function_arguments);

        let invocation = prepare_task(&staged_task);

        let worker = Worker::default();
        let result = worker.invoke_function(invocation);
        if result.is_ok() {
            finalize_task(&staged_task).unwrap();
        }
        assert_eq!(result.unwrap(), "Hello, Teaclave!");
    }

    pub fn test_invoke_gbdt_training() {
        let task_id = Uuid::new_v4();
        let function_arguments = FunctionArguments::new(hashmap!(
            "feature_size"                => "4",
            "max_depth"                   => "4",
            "iterations"                  => "100",
            "shrinkage"                   => "0.1",
            "feature_sample_ratio"        => "1.0",
            "data_sample_ratio"           => "1.0",
            "min_leaf_size"               => "1",
            "loss"                        => "LAD",
            "training_optimization_level" => "2",
        ));
        let fixture_dir = format!(
            "file:///{}/fixtures/functions/gbdt_training",
            env!("TEACLAVE_TEST_INSTALL_DIR")
        );
        let input_url = Url::parse(&format!("{}/train.enc", fixture_dir)).unwrap();
        let output_url = Url::parse(&format!(
            "{}/model-{}.enc.out",
            fixture_dir,
            task_id.to_string()
        ))
        .unwrap();
        let crypto = TeaclaveFile128Key::new(&[0; 16]).unwrap();
        let crypto_info = FileCrypto::TeaclaveFile128(crypto);

        let training_input_data = FunctionInputFile::new(input_url, "", crypto_info);
        let model_output_data = FunctionOutputFile::new(output_url, crypto_info);

        let input_data = hashmap!("training_data" => training_input_data);
        let output_data = hashmap!("trained_model" => model_output_data);

        let staged_task = StagedTask::new()
            .task_id(task_id)
            .function_name("gbdt_training")
            .function_arguments(function_arguments)
            .input_data(input_data)
            .output_data(output_data);

        let invocation = prepare_task(&staged_task);

        let worker = Worker::default();
        let result = worker.invoke_function(invocation);
        if result.is_ok() {
            finalize_task(&staged_task).unwrap();
        }
        log::debug!("summary: {:?}", result);
        assert!(result.is_ok());
    }
}
