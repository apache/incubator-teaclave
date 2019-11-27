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
use crate::running_task::RunningTask;
use crate::trusted_worker::{
    BytesPlusOneWorker, ConcatWorker, DBSCANWorker, EchoFileWorker, EchoWorker,
    FileBytesPlusOneWorker, GBDTPredictWorker, GBDTTrainWorker, GPWorker, GenLinearModelWorker,
    GmmWorker, ImageResizeWorker, KmeansWorker, LinRegWorker, LogisticRegPredictWorker,
    LogisticRegTrainWorker, MesaPyWorker, NaiveBayesWorker, NeuralNetWorker, OnlineDecryptWorker,
    PSIWorker, PrivateJoinAndComputeWorker, RSASignWorker, SvmWorker, SwapFileWorker, WASMWorker,
};
use crate::worker::WorkerInfoQueue;
use mesatee_core::Result;
use sgx_types::{c_char, c_int, size_t};
use std::ffi::CStr;
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use std::slice;

const FFI_FILE_ERROR: c_int = -1;
const FFI_BUFFER_NOT_ENOUGH_ERROR: c_int = -2;
const UUID_SIZE: size_t = 36;

// C API of read_file for workers
//
// int c_read_file(char* context_id,
//                 char* context_token,
//                 char* file_id,
//                 char* out_buf,
//                 size_t* out_buf_size);
#[allow(unused)]
#[no_mangle]
extern "C" fn c_read_file(
    context_id: *const c_char,
    context_token: *const c_char,
    file_id: *const c_char,
    out_buf: *mut u8,
    out_buf_size_p: *mut size_t,
) -> c_int {
    let context_id = unsafe { CStr::from_ptr(context_id).to_string_lossy().into_owned() };
    let context_token = unsafe { CStr::from_ptr(context_token).to_string_lossy().into_owned() };
    let file_id = unsafe { CStr::from_ptr(file_id).to_string_lossy().into_owned() };
    let out_buf_size = unsafe { *out_buf_size_p };
    let out: &mut [u8] = unsafe { slice::from_raw_parts_mut(out_buf, out_buf_size) };

    match read_file(&context_id, &context_token, &file_id) {
        Ok(content) => {
            let content_len = content.len();
            if content_len <= out_buf_size {
                out[..content.len()].copy_from_slice(&content);
                content_len as c_int
            } else {
                unsafe { *out_buf_size_p = content_len }
                FFI_BUFFER_NOT_ENOUGH_ERROR
            }
        }
        Err(_) => FFI_FILE_ERROR,
    }
}

// C API of save_file_for_task_creator for workers
//
// int c_save_file_for_task_creator(char* context_id,
//                                  char* context_token,
//                                  char* in_buf,
//                                  size_t in_buf_size,
//                                  char* out_file_id_buf,
//                                  size_t out_file_id_buf_size);
#[allow(unused)]
#[no_mangle]
extern "C" fn c_save_file_for_task_creator(
    context_id: *const c_char,
    context_token: *const c_char,
    in_buf: *const u8,
    in_buf_size: size_t,
    out_file_id_buf: *mut u8,
    out_file_id_buf_size: size_t,
) -> c_int {
    if out_file_id_buf_size < UUID_SIZE {
        return FFI_BUFFER_NOT_ENOUGH_ERROR;
    }
    let context_id = unsafe { CStr::from_ptr(context_id).to_string_lossy().into_owned() };
    let context_token = unsafe { CStr::from_ptr(context_token).to_string_lossy().into_owned() };
    let in_buf: &[u8] = unsafe { slice::from_raw_parts(in_buf, in_buf_size) };
    let out_file_id: &mut [u8] =
        unsafe { slice::from_raw_parts_mut(out_file_id_buf, out_file_id_buf_size) };

    match save_file_for_task_creator(&context_id, &context_token, in_buf) {
        Ok(file_id) => {
            let file_id_len = file_id.len();
            if file_id_len <= out_file_id_buf_size {
                out_file_id[..file_id_len].copy_from_slice(file_id.as_bytes());
                file_id_len as c_int
            } else {
                FFI_BUFFER_NOT_ENOUGH_ERROR
            }
        }
        Err(_) => FFI_FILE_ERROR,
    }
}

// C API of save_file_for_all_participants for workers
//
// int c_save_file_for_all_participants(char* context_id,
//                                      char* context_token,
//                                      char* in_buf,
//                                      size_t in_buf_size,
//                                      char* out_file_id_buf,
//                                      size_t out_file_id_buf_size);
#[allow(unused)]
#[no_mangle]
extern "C" fn c_save_file_for_all_participants(
    context_id: *const c_char,
    context_token: *const c_char,
    in_buf: *const u8,
    in_buf_size: size_t,
    out_file_id_buf: *mut u8,
    out_file_id_buf_size: size_t,
) -> c_int {
    if out_file_id_buf_size < UUID_SIZE {
        return FFI_BUFFER_NOT_ENOUGH_ERROR;
    }
    let context_id = unsafe { CStr::from_ptr(context_id).to_string_lossy().into_owned() };
    let context_token = unsafe { CStr::from_ptr(context_token).to_string_lossy().into_owned() };
    let in_buf: &[u8] = unsafe { slice::from_raw_parts(in_buf, in_buf_size) };
    let out_file_id: &mut [u8] =
        unsafe { slice::from_raw_parts_mut(out_file_id_buf, out_file_id_buf_size) };

    match save_file_for_all_participants(&context_id, &context_token, in_buf) {
        Ok(file_id) => {
            let file_id_len = file_id.len();
            if file_id_len <= out_file_id_buf_size {
                out_file_id[..file_id_len].copy_from_slice(file_id.as_bytes());
                file_id_len as c_int
            } else {
                FFI_BUFFER_NOT_ENOUGH_ERROR
            }
        }
        Err(_) => FFI_FILE_ERROR,
    }
}

// C API of save_file_for_file_owner for workers
//
// int c_save_file_for_file_owner(char* context_id,
//                                char* context_token,
//                                char* in_buf,
//                                size_t in_buf_size,
//                                char* file_id,
//                                char* out_file_id_buf,
//                                size_t out_file_id_buf_size)
#[allow(unused)]
#[no_mangle]
extern "C" fn c_save_file_for_file_owner(
    context_id: *const c_char,
    context_token: *const c_char,
    in_buf: *const u8,
    in_buf_size: size_t,
    in_file_id: *const c_char,
    out_file_id_buf: *mut u8,
    out_file_id_buf_size: size_t,
) -> c_int {
    if out_file_id_buf_size < UUID_SIZE {
        return FFI_BUFFER_NOT_ENOUGH_ERROR;
    }
    let context_id = unsafe { CStr::from_ptr(context_id).to_string_lossy().into_owned() };
    let context_token = unsafe { CStr::from_ptr(context_token).to_string_lossy().into_owned() };
    let in_buf: &[u8] = unsafe { slice::from_raw_parts(in_buf, in_buf_size) };
    let in_file_id = unsafe { CStr::from_ptr(in_file_id).to_string_lossy().into_owned() };
    let out_file_id: &mut [u8] =
        unsafe { slice::from_raw_parts_mut(out_file_id_buf, out_file_id_buf_size) };

    match save_file_for_file_owner(&context_id, &context_token, in_buf, &in_file_id) {
        Ok(file_id) => {
            let file_id_len = file_id.len();
            if file_id_len <= out_file_id_buf_size {
                out_file_id[..file_id_len].copy_from_slice(file_id.as_bytes());
                file_id_len as c_int
            } else {
                FFI_BUFFER_NOT_ENOUGH_ERROR
            }
        }
        Err(_) => FFI_FILE_ERROR,
    }
}

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
    for _i in 0..10 {
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

        let worker = Box::new(GBDTTrainWorker::new());
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

        let worker = Box::new(SvmWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(GenLinearModelWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(GmmWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(GPWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(DBSCANWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(NeuralNetWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(NaiveBayesWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(LogisticRegTrainWorker::new());
        let _ = WorkerInfoQueue::register(worker);

        let worker = Box::new(LogisticRegPredictWorker::new());
        let _ = WorkerInfoQueue::register(worker);
    }
}
