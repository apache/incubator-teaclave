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

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use std::sync::{Arc, SgxMutex as Mutex};

use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::{StagedTask, WorkerInvocationResult};
use teaclave_worker::Worker;

use anyhow::Result;

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
                    log::error!("Error: {:?}", e);
                    continue;
                }
            };

            let request = PullTaskRequest {};
            log::debug!("pull_task");
            let response = match client.pull_task(request) {
                Ok(response) => response,
                Err(e) => {
                    log::error!("Error: {:?}", e);
                    continue;
                }
            };
            log::debug!("response: {:?}", response);
            let result = self.invoke_task(response.staged_task);
            self.update_task(result);
        }
    }

    fn invoke_task(&mut self, _task: StagedTask) -> WorkerInvocationResult {
        // TODO: convert task to function, i.e., needs help from agent
        unimplemented!()
    }

    fn update_task(&mut self, _result: WorkerInvocationResult) {
        unimplemented!()
    }
}

#[cfg(test_mode)]
mod test_mode {
    use super::*;
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use std::convert::TryInto;
    use teaclave_types::*;

    pub fn test_invoke_function() {
        let function_args = TeaclaveFunctionArguments::new(&hashmap!(
            "feature_size"  => "4",
            "max_depth"     => "4",
            "iterations"    => "100",
            "shrinkage"     => "0.1",
            "feature_sample_ratio" => "1.0",
            "data_sample_ratio" => "1.0",
            "min_leaf_size" => "1",
            "loss"          => "LAD",
            "training_optimization_level" => "2"
        ));

        let plain_input = "fixtures/functions/gbdt_training/train.txt";
        let enc_output = "fixtures/functions/gbdt_training/model.enc.out";

        let input_info =
            TeaclaveWorkerInputFileInfo::create_with_plaintext_file(plain_input).unwrap();
        let input_files = TeaclaveWorkerFileRegistry::new(hashmap!(
            "training_data".to_string() => input_info));

        let output_info =
            TeaclaveWorkerOutputFileInfo::new(enc_output, TeaclaveFileRootKey128::default());
        let output_files = TeaclaveWorkerFileRegistry::new(hashmap!(
            "trained_model".to_string() => output_info));
        let invocation = WorkerInvocation {
            runtime_name: "default".to_string(),
            executor_type: "native".try_into().unwrap(),
            function_name: "gbdt_training".to_string(),
            function_payload: String::new(),
            function_args,
            input_files,
            output_files,
        };

        let worker = Worker::default();
        let result = worker.invoke_function(invocation);
        assert!(result.is_ok());
        log::debug!("summary: {:?}", result.unwrap());
    }

    pub fn test_invoke_echo_function() {
        let invocation = WorkerInvocation {
            runtime_name: "default".to_string(),
            executor_type: "native".try_into().unwrap(),
            function_name: "echo".to_string(),
            function_payload: String::new(),
            function_args: TeaclaveFunctionArguments::new(&hashmap!(
                "payload" => "Hello Teaclave!"
            )),
            input_files: TeaclaveWorkerFileRegistry::default(),
            output_files: TeaclaveWorkerFileRegistry::default(),
        };

        let worker = Worker::default();
        let result = worker.invoke_function(invocation).unwrap();
        assert_eq!(result, "Hello Teaclave!");
    }
}
