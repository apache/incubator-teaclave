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
use crate::global;
use lazy_static::lazy_static;
use mesatee_core::{Error, ErrorKind, Result};
use std::collections::{HashMap, HashSet};
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;
#[cfg(feature = "mesalock_sgx")]
use std::sync::SgxRwLock as RwLock;
pub use tms_internal_proto::FunctionType;

pub struct WorkerContext {
    pub context_id: String, // Context_id and context_token are used for retrieving RunningTask
    pub context_token: String,
}

impl WorkerContext {
    pub fn read_file(&self, file_id: &str) -> Result<Vec<u8>> {
        global::read_file(&self.context_id, &self.context_token, file_id)
    }
    pub fn save_file_for_file_owner(&self, data: &[u8], file_id: &str) -> Result<String> {
        global::save_file_for_file_owner(&self.context_id, &self.context_token, data, file_id)
    }
    #[allow(dead_code)]
    pub fn save_file_for_task_creator(&self, data: &[u8]) -> Result<String> {
        global::save_file_for_task_creator(&self.context_id, &self.context_token, data)
    }
    pub fn save_file_for_all_participants(&self, data: &[u8]) -> Result<String> {
        global::save_file_for_all_participants(&self.context_id, &self.context_token, data)
    }
}

pub trait Worker: Send + Sync {
    fn function_name(&self) -> &str;
    fn function_type(&self) -> FunctionType;
    fn set_id(&mut self, worker_id: u32);
    fn id(&self) -> u32;
    fn prepare_input(&mut self, dynamic_input: Option<String>, file_ids: Vec<String>)
        -> Result<()>;
    fn execute(&mut self, context: WorkerContext) -> Result<String>;
}

pub struct WorkerInfoQueue {
    running_worker: HashSet<u32>,
    worker_id_counter: u32,
    queue: HashMap<String, Vec<Box<dyn Worker>>>,
}

lazy_static! {
    static ref WORKER_INFO_QUEUE: RwLock<WorkerInfoQueue> = RwLock::new(WorkerInfoQueue::new());
}

impl WorkerInfoQueue {
    pub fn new() -> Self {
        WorkerInfoQueue {
            running_worker: HashSet::new(),
            worker_id_counter: 0,
            queue: HashMap::new(),
        }
    }

    pub fn inc_id(&mut self) -> u32 {
        let id = self.worker_id_counter;
        self.worker_id_counter += 1;
        id
    }

    pub fn register(mut worker: Box<dyn Worker>) -> Result<()> {
        let mut worker_info_queue = WORKER_INFO_QUEUE.write()?;
        let worker_id = worker_info_queue.inc_id();

        let func_name = worker.function_name().to_owned();

        if !worker_info_queue.queue.contains_key(&func_name) {
            worker_info_queue
                .queue
                .insert(func_name.to_owned(), Vec::new());
        }
        let queue = worker_info_queue
            .queue
            .get_mut(&func_name)
            .ok_or_else(|| Error::from(ErrorKind::BadImplementation))?;

        worker.set_id(worker_id);
        queue.push(worker);
        Ok(())
    }

    pub fn aquire_worker(func_name: &str) -> Result<Box<dyn Worker>> {
        let mut worker_info_queue = WORKER_INFO_QUEUE.write()?;
        let queue = worker_info_queue
            .queue
            .get_mut(&func_name.to_string())
            .ok_or_else(|| Error::from(ErrorKind::FunctionNotSupportedError))?;

        match queue.pop() {
            Some(worker) => {
                let worker_id = worker.id();
                worker_info_queue.running_worker.insert(worker_id);
                Ok(worker)
            }
            None => Err(Error::from(ErrorKind::NoValidWorkerError)),
        }
    }

    pub fn release_worker(worker: Box<dyn Worker>) -> Result<()> {
        let mut worker_info_queue = WORKER_INFO_QUEUE.write()?;
        let worker_id = worker.id();
        worker_info_queue.running_worker.remove(&worker_id);

        let queue = worker_info_queue
            .queue
            .get_mut(&worker.function_name().to_string())
            .ok_or_else(|| Error::from(ErrorKind::BadImplementation))?;

        queue.push(worker);
        Ok(())
    }
}
