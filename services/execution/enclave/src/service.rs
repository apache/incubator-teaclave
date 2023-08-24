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

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::task_file_manager::TaskFileManager;
use anyhow::Result;
use teaclave_proto::teaclave_common::{ExecutorCommand, ExecutorStatus};
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_rpc::transport::{channel::Endpoint, Channel};
use teaclave_types::*;
use teaclave_worker::Worker;
use uuid::Uuid;

static WORKER_BASE_DIR: &str = "/tmp/teaclave_agent/";

#[derive(Clone)]
pub(crate) struct TeaclaveExecutionService {
    #[allow(dead_code)]
    worker: Arc<Worker>,
    scheduler_client: TeaclaveSchedulerClient<Channel>,
    fusion_base: PathBuf,
    id: Uuid,
    status: ExecutorStatus,
}

impl TeaclaveExecutionService {
    pub(crate) async fn new(
        scheduler_service_endpoint: Endpoint,
        fusion_base: impl AsRef<Path>,
    ) -> Result<Self> {
        let channel = scheduler_service_endpoint.connect().await?;
        let scheduler_client = TeaclaveSchedulerClient::new_with_builtin_config(channel);

        Ok(TeaclaveExecutionService {
            worker: Arc::new(Worker::default()),
            scheduler_client,
            fusion_base: fusion_base.as_ref().to_owned(),
            id: Uuid::new_v4(),
            status: ExecutorStatus::Idle,
        })
    }

    pub(crate) async fn start(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        let mut current_task: Arc<Option<StagedTask>> = Arc::new(None);
        let mut task_handle: Option<thread::JoinHandle<()>> = None;

        loop {
            std::thread::sleep(std::time::Duration::from_secs(3));

            match self.heartbeat().await {
                Ok(ExecutorCommand::Stop) => {
                    log::info!("Executor {} is stopped", self.id);
                    return Err(anyhow::anyhow!("EnclaveForceTermination"));
                }
                Ok(ExecutorCommand::NewTask) if self.status == ExecutorStatus::Idle => {
                    match self.pull_task().await {
                        Ok(task) => {
                            self.status = ExecutorStatus::Executing;
                            self.update_task_status(&task.task_id, TaskStatus::Running)
                                .await?;
                            let tx_task = tx.clone();
                            let fusion_base = self.fusion_base.clone();
                            current_task = Arc::new(Some(task));
                            let task_copy = current_task.clone();
                            let handle = thread::spawn(move || {
                                let result =
                                    invoke_task(task_copy.as_ref().as_ref().unwrap(), &fusion_base);
                                tx_task.send(result).unwrap();
                            });
                            task_handle = Some(handle);
                        }
                        Err(e) => {
                            log::error!("Executor {} failed to pull task: {}", self.id, e);
                        }
                    };
                }
                Err(e) => {
                    log::error!("Executor {} failed to heartbeat: {}", self.id, e);
                    return Err(e);
                }
                _ => {}
            }

            match rx.try_recv() {
                Ok(result) => {
                    let task_unwrapped = current_task.as_ref().as_ref().unwrap();
                    match result {
                        Ok(_) => log::debug!(
                            "InvokeTask: {:?}, {:?}, success",
                            task_unwrapped.task_id,
                            task_unwrapped.function_id
                        ),
                        Err(_) => log::debug!(
                            "InvokeTask: {:?}, {:?}, failure",
                            task_unwrapped.task_id,
                            task_unwrapped.function_id
                        ),
                    }
                    log::debug!("InvokeTask result: {:?}", result);
                    let task_copy = current_task.clone();
                    match self
                        .update_task_result(&task_copy.as_ref().as_ref().unwrap().task_id, result)
                        .await
                    {
                        Ok(_) => (),
                        Err(e) => {
                            log::error!("UpdateResult Error: {:?}", e);
                            continue;
                        }
                    }
                    current_task = Arc::new(None);
                    task_handle.unwrap().join().unwrap();
                    task_handle = None;
                    self.status = ExecutorStatus::Idle;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    log::error!(
                        "Executor {} failed to receive, sender disconnected",
                        self.id
                    );
                }
                // received nothing
                Err(_) => {}
            }
        }
    }

    async fn pull_task(&mut self) -> Result<StagedTask> {
        let request = PullTaskRequest {
            executor_id: self.id.to_string(),
        };
        let response = self.scheduler_client.pull_task(request).await?.into_inner();

        log::debug!("pull_stask response: {:?}", response);
        let staged_task = StagedTask::from_slice(&response.staged_task)?;
        Ok(staged_task)
    }

    async fn heartbeat(&mut self) -> Result<ExecutorCommand> {
        let request = HeartbeatRequest::new(self.id, self.status);
        let response = self.scheduler_client.heartbeat(request).await?.into_inner();

        log::debug!("heartbeat_with_result response: {:?}", response);
        response.command.try_into()
    }

    async fn update_task_result(
        &mut self,
        task_id: &Uuid,
        task_result: Result<TaskOutputs>,
    ) -> Result<()> {
        let request = UpdateTaskResultRequest::new(*task_id, task_result);

        let _response = self.scheduler_client.update_task_result(request).await?;

        Ok(())
    }

    async fn update_task_status(&mut self, task_id: &Uuid, task_status: TaskStatus) -> Result<()> {
        let request = UpdateTaskStatusRequest::new(task_id.to_owned(), task_status);
        let _response = self.scheduler_client.update_task_status(request).await?;

        Ok(())
    }
}

fn invoke_task(task: &StagedTask, fusion_base: &PathBuf) -> Result<TaskOutputs> {
    let save_log = task
        .function_arguments
        .get("save_log")
        .ok()
        .and_then(|v| v.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(false);
    let log_arc = Arc::new(Mutex::new(Vec::<String>::new()));

    if save_log {
        let log_arc = Arc::into_raw(log_arc.clone());
        log::info!(buffer = log_arc.expose_addr(); "");
    }

    let file_mgr = TaskFileManager::new(
        WORKER_BASE_DIR,
        fusion_base,
        &task.task_id,
        &task.input_data,
        &task.output_data,
    )?;
    let invocation = prepare_task(task, &file_mgr)?;

    log::debug!("Invoke function: {:?}", invocation);
    let worker = Worker::default();
    let summary = worker.invoke_function(invocation)?;

    let outputs_tag = finalize_task(&file_mgr)?;
    if save_log {
        log::info!(buffer = 0; "");
    }

    let log = Arc::try_unwrap(log_arc)
        .map_err(|_| anyhow::anyhow!("log buffer is referenced more than once"))?
        .into_inner()?;
    let task_outputs = TaskOutputs::new(summary.as_bytes(), outputs_tag, log);

    Ok(task_outputs)
}

fn prepare_task(task: &StagedTask, file_mgr: &TaskFileManager) -> Result<StagedFunction> {
    let input_files = file_mgr.prepare_staged_inputs()?;
    let output_files = file_mgr.prepare_staged_outputs()?;

    let staged_function = StagedFunctionBuilder::new()
        .executor_type(task.executor_type)
        .executor(task.executor)
        .name(&task.function_name)
        .arguments(task.function_arguments.clone())
        .payload(task.function_payload.clone())
        .input_files(input_files)
        .output_files(output_files)
        .runtime_name("default")
        .build();
    Ok(staged_function)
}

fn finalize_task(file_mgr: &TaskFileManager) -> Result<HashMap<String, FileAuthTag>> {
    file_mgr.upload_outputs()
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use serde_json::json;
    use std::format;
    use teaclave_crypto::*;
    use url::Url;
    use uuid::Uuid;

    pub fn test_invoke_echo() {
        let task_id = Uuid::new_v4();
        let function_arguments =
            FunctionArguments::from_json(json!({"message": "Hello, Teaclave!"})).unwrap();
        let staged_task = StagedTaskBuilder::new()
            .task_id(task_id)
            .executor(Executor::Builtin)
            .function_name("builtin-echo")
            .function_arguments(function_arguments)
            .build();

        let file_mgr = TaskFileManager::new(
            WORKER_BASE_DIR,
            "/tmp/fusion_base",
            &staged_task.task_id,
            &staged_task.input_data,
            &staged_task.output_data,
        )
        .unwrap();
        let invocation = prepare_task(&staged_task, &file_mgr).unwrap();

        let worker = Worker::default();
        let result = worker.invoke_function(invocation);
        if result.is_ok() {
            finalize_task(&file_mgr).unwrap();
        }
        assert_eq!(result.unwrap(), "Hello, Teaclave!");
    }

    pub fn test_invoke_gbdt_train() {
        let task_id = Uuid::new_v4();
        let function_arguments = FunctionArguments::from_json(json!({
            "feature_size": 4,
            "max_depth": 4,
            "iterations": 100,
            "shrinkage": 0.1,
            "feature_sample_ratio": 1.0,
            "data_sample_ratio": 1.0,
            "min_leaf_size": 1,
            "loss": "LAD",
            "training_optimization_level": 2,
        }))
        .unwrap();
        let fixture_dir = format!(
            "file:///{}/fixtures/functions/gbdt_training",
            env!("TEACLAVE_TEST_INSTALL_DIR")
        );
        let input_url = Url::parse(&format!("{}/train.enc", fixture_dir)).unwrap();
        let output_url = Url::parse(&format!("{}/model-{}.enc.out", fixture_dir, task_id)).unwrap();
        let crypto = TeaclaveFile128Key::new(&[0; 16]).unwrap();
        let input_cmac = FileAuthTag::from_hex("860030495909b84864b991865e9ad94f").unwrap();
        let training_input_data = FunctionInputFile::new(input_url, input_cmac, crypto);
        let model_output_data = FunctionOutputFile::new(output_url, crypto);

        let input_data = hashmap!("training_data" => training_input_data);
        let output_data = hashmap!("trained_model" => model_output_data);

        let staged_task = StagedTaskBuilder::new()
            .task_id(task_id)
            .executor(Executor::Builtin)
            .function_name("builtin-gbdt-train")
            .function_arguments(function_arguments)
            .input_data(input_data)
            .output_data(output_data)
            .build();

        let file_mgr = TaskFileManager::new(
            WORKER_BASE_DIR,
            "/tmp/fusion_base",
            &staged_task.task_id,
            &staged_task.input_data,
            &staged_task.output_data,
        )
        .unwrap();
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
