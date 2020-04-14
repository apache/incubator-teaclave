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

use std::prelude::v1::*;
use std::sync::{Arc, SgxMutex as Mutex};

use crate::task_file_manager::TaskFileManager;
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::*;
use teaclave_worker::Worker;

use anyhow::Result;
use uuid::Uuid;

static WORKER_BASE_DIR: &str = "/tmp/teaclave_agent/";

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
                    anyhow::ensure!(i < 10, "failed to connect to scheduler service");
                    log::debug!("Failed to connect to scheduler service, retry {}", i);
                    i += 1;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(3));
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
            let staged_task = match self.pull_task() {
                Ok(staged_task) => staged_task,
                Err(e) => {
                    log::error!("PullTask Error: {:?}", e);
                    continue;
                }
            };

            log::info!("InvokeTask: {:?}", staged_task);
            let result = self.invoke_task(&staged_task);
            log::info!("InvokeTask result: {:?}", result);

            match self.update_task_result(&staged_task.task_id, result) {
                Ok(_) => (),
                Err(e) => {
                    log::error!("UpdateResult Error: {:?}", e);
                    continue;
                }
            }
        }
    }

    fn pull_task(&mut self) -> Result<StagedTask> {
        let request = PullTaskRequest {};
        let response = self
            .scheduler_client
            .clone()
            .lock()
            .map_err(|_| anyhow::anyhow!("Cannot lock scheduler client"))?
            .pull_task(request)?;

        log::debug!("pull_stask response: {:?}", response);
        Ok(response.staged_task)
    }

    fn invoke_task(&mut self, task: &StagedTask) -> Result<TaskOutputs> {
        let file_mgr = TaskFileManager::new(WORKER_BASE_DIR, task)?;

        self.update_task_status(&task.task_id, TaskStatus::DataPreparing, String::new())?;
        let invocation = prepare_task(&task, &file_mgr)?;

        log::info!("Invoke function: {:?}", invocation);

        self.update_task_status(&task.task_id, TaskStatus::Running, String::new())?;
        let worker = Worker::default();
        let summary = worker.invoke_function(invocation)?;

        finalize_task(&file_mgr)?;
        let task_outputs = TaskOutputs::new(summary.as_bytes(), hashmap!());
        Ok(task_outputs)
    }

    fn update_task_result(
        &mut self,
        task_id: &Uuid,
        task_result: Result<TaskOutputs>,
    ) -> Result<()> {
        let request = UpdateTaskResultRequest::new(*task_id, task_result);

        let _response = self
            .scheduler_client
            .clone()
            .lock()
            .map_err(|_| anyhow::anyhow!("Cannot lock scheduler client"))?
            .update_task_result(request)?;

        Ok(())
    }

    fn update_task_status(
        &mut self,
        task_id: &Uuid,
        task_status: TaskStatus,
        status_info: String,
    ) -> Result<()> {
        let request = UpdateTaskStatusRequest::new(task_id.to_owned(), task_status, status_info);
        let _response = self
            .scheduler_client
            .clone()
            .lock()
            .map_err(|_| anyhow::anyhow!("Cannot lock scheduler client"))?
            .update_task_status(request)?;

        Ok(())
    }
}

fn prepare_task(task: &StagedTask, file_mgr: &TaskFileManager) -> Result<StagedFunction> {
    file_mgr.download_inputs()?;
    let input_files = file_mgr.convert_downloaded_inputs()?;
    let output_files = file_mgr.prepare_staged_outputs()?;
    let function_payload = String::from_utf8_lossy(&task.function_payload).to_string();

    let staged_function = StagedFunction::new()
        .executor_type(task.executor_type)
        .executor(task.executor)
        .payload(function_payload)
        .arguments(task.function_arguments.clone())
        .input_files(input_files)
        .output_files(output_files)
        .runtime_name("default");
    Ok(staged_function)
}

fn finalize_task(file_mgr: &TaskFileManager) -> Result<()> {
    file_mgr.upload_outputs()
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use std::format;
    use teaclave_crypto::*;
    use url::Url;
    use uuid::Uuid;

    pub fn test_invoke_echo() {
        let task_id = Uuid::new_v4();
        let staged_task = StagedTask::new()
            .task_id(task_id)
            .executor(Executor::Echo)
            .function_arguments(hashmap!("message" => "Hello, Teaclave!"));

        let file_mgr = TaskFileManager::new(WORKER_BASE_DIR, &staged_task).unwrap();
        let invocation = prepare_task(&staged_task, &file_mgr).unwrap();

        let worker = Worker::default();
        let result = worker.invoke_function(invocation);
        if result.is_ok() {
            finalize_task(&file_mgr).unwrap();
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

        let training_input_data =
            FunctionInputFile::new(input_url, FileAuthTag::mock(), crypto_info);
        let model_output_data = FunctionOutputFile::new(output_url, crypto_info);

        let input_data = hashmap!("training_data" => training_input_data);
        let output_data = hashmap!("trained_model" => model_output_data);

        let staged_task = StagedTask::new()
            .task_id(task_id)
            .executor(Executor::GbdtTraining)
            .function_arguments(function_arguments)
            .input_data(input_data)
            .output_data(output_data);

        let file_mgr = TaskFileManager::new(WORKER_BASE_DIR, &staged_task).unwrap();
        let invocation = prepare_task(&staged_task, &file_mgr).unwrap();

        let worker = Worker::default();
        let result = worker.invoke_function(invocation);
        if result.is_ok() {
            finalize_task(&file_mgr).unwrap();
        }
        log::debug!("summary: {:?}", result);
        assert!(result.is_ok());
    }
}
