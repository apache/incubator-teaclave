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
use crate::running_task::RunningTask;
use crate::trusted_worker::{
    BytesPlusOneWorker, ConcatWorker, EchoFileWorker, EchoWorker, FileBytesPlusOneWorker,
    GBDTPredictWorker, GenLinearModelWorker, ImageResizeWorker, KmeansWorker, LinRegWorker,
    LogisticRegWorker, MesaPyWorker, OnlineDecryptWorker, PSIWorker, PrivateJoinAndComputeWorker,
    RSASignWorker, SvmWorker, SwapFileWorker, WASMWorker,
};
use crate::worker::WorkerInfoQueue;
use mesatee_core::Result;
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

pub fn read_file(context_id: &str, context_token: &str, file_id: &str) -> Result<Vec<u8>> {
    let mut running_task = RunningTask::retrieve_running_task(context_id, context_token)?;
    running_task.read_file(file_id)
}

pub fn save_file_for_task_creator(
    context_id: &str,
    context_token: &str,
    data: &[u8],
) -> Result<String> {
    let mut running_task = RunningTask::retrieve_running_task(context_id, context_token)?;
    running_task.save_file_for_task_creator(data)
}

pub fn save_file_for_all_participants(
    context_id: &str,
    context_token: &str,
    data: &[u8],
) -> Result<String> {
    let mut running_task = RunningTask::retrieve_running_task(context_id, context_token)?;
    running_task.save_file_for_all_participants(data)
}
pub fn save_file_for_file_owner(
    context_id: &str,
    context_token: &str,
    data: &[u8],
    file_id: &str,
) -> Result<String> {
    let mut running_task = RunningTask::retrieve_running_task(context_id, context_token)?;
    running_task.save_file_for_file_owner(data, file_id)
}

pub fn register_trusted_worker_statically() {
    for _i in 0..18 {
        let worker = Box::new(EchoWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(PSIWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(EchoFileWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(BytesPlusOneWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(FileBytesPlusOneWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(ConcatWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(SwapFileWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(WASMWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(MesaPyWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(GBDTPredictWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(PrivateJoinAndComputeWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(ImageResizeWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(OnlineDecryptWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(RSASignWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(KmeansWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(LinRegWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(LogisticRegWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(SvmWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(GenLinearModelWorker::new());
        let _ = WorkerInfoQueue::register(worker);
    }
}
