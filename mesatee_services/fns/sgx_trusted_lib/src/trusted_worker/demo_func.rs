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

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::worker::{FunctionType, Worker, WorkerContext};
use mesatee_core::{Error, ErrorKind, Result};

pub struct EchoWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<EchoWorkerInput>,
}
impl EchoWorker {
    pub fn new() -> Self {
        EchoWorker {
            worker_id: 0,
            func_name: "echo".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}
struct EchoWorkerInput {
    msg: String,
}
impl Worker for EchoWorker {
    fn function_name(&self) -> &str {
        self.func_name.as_str()
    }
    fn function_type(&self) -> FunctionType {
        self.func_type
    }
    fn set_id(&mut self, worker_id: u32) {
        self.worker_id = worker_id;
    }
    fn id(&self) -> u32 {
        self.worker_id
    }
    fn prepare_input(
        &mut self,
        dynamic_input: Option<String>,
        _file_ids: Vec<String>,
    ) -> Result<()> {
        let msg = dynamic_input.ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        self.input = Some(EchoWorkerInput { msg });
        Ok(())
    }
    fn execute(&mut self, _context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        Ok(input.msg)
    }
}

pub struct EchoFileWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<EchoFileWorkerInput>,
}
impl EchoFileWorker {
    pub fn new() -> Self {
        EchoFileWorker {
            worker_id: 0,
            func_name: "echo_file".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}
struct EchoFileWorkerInput {
    file_id: String,
}
impl Worker for EchoFileWorker {
    fn function_name(&self) -> &str {
        self.func_name.as_str()
    }
    fn function_type(&self) -> FunctionType {
        self.func_type
    }
    fn set_id(&mut self, worker_id: u32) {
        self.worker_id = worker_id;
    }
    fn id(&self) -> u32 {
        self.worker_id
    }
    fn prepare_input(
        &mut self,
        dynamic_input: Option<String>,
        _file_ids: Vec<String>,
    ) -> Result<()> {
        let file_id = dynamic_input.ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        self.input = Some(EchoFileWorkerInput { file_id });
        Ok(())
    }
    fn execute(&mut self, context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        let bytes: Vec<u8> = context.read_file(&input.file_id)?;
        let result = String::from_utf8_lossy(&bytes).to_string();
        Ok(result)
    }
}

pub struct BytesPlusOneWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<BytesPlusOneWorkerInput>,
}
impl BytesPlusOneWorker {
    pub fn new() -> Self {
        BytesPlusOneWorker {
            worker_id: 0,
            func_name: "bytes_plus_one".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}
struct BytesPlusOneWorkerInput {
    msg: String,
}
impl Worker for BytesPlusOneWorker {
    fn function_name(&self) -> &str {
        self.func_name.as_str()
    }
    fn function_type(&self) -> FunctionType {
        self.func_type
    }
    fn set_id(&mut self, worker_id: u32) {
        self.worker_id = worker_id;
    }
    fn id(&self) -> u32 {
        self.worker_id
    }
    fn prepare_input(
        &mut self,
        dynamic_input: Option<String>,
        _file_ids: Vec<String>,
    ) -> Result<()> {
        let msg = dynamic_input.ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        self.input = Some(BytesPlusOneWorkerInput { msg });
        Ok(())
    }
    fn execute(&mut self, _context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        let bytes: Vec<u8> = input.msg.as_bytes().iter().map(|x| x + 1).collect();
        let result = String::from_utf8_lossy(&bytes).to_string();
        Ok(result)
    }
}

pub struct FileBytesPlusOneWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<FileBytesPlusOneWorkerInput>,
}
impl FileBytesPlusOneWorker {
    pub fn new() -> Self {
        FileBytesPlusOneWorker {
            worker_id: 0,
            func_name: "file_bytes_plus_one".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}
struct FileBytesPlusOneWorkerInput {
    file_id: String,
}

impl Worker for FileBytesPlusOneWorker {
    fn function_name(&self) -> &str {
        self.func_name.as_str()
    }
    fn function_type(&self) -> FunctionType {
        self.func_type
    }
    fn set_id(&mut self, worker_id: u32) {
        self.worker_id = worker_id;
    }
    fn id(&self) -> u32 {
        self.worker_id
    }
    fn prepare_input(
        &mut self,
        dynamic_input: Option<String>,
        _file_ids: Vec<String>,
    ) -> Result<()> {
        let file_id = dynamic_input.ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        self.input = Some(FileBytesPlusOneWorkerInput { file_id });
        Ok(())
    }
    fn execute(&mut self, context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        let bytes: Vec<u8> = context.read_file(&input.file_id)?;
        let modified_bytes: Vec<u8> = bytes.iter().map(|x| x + 1).collect();
        let result = String::from_utf8_lossy(&modified_bytes).to_string();
        Ok(result)
    }
}

pub struct ConcatWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<ConcatWorkerInput>,
}
impl ConcatWorker {
    pub fn new() -> Self {
        ConcatWorker {
            worker_id: 0,
            func_name: "concat".to_string(),
            func_type: FunctionType::Multiparty,
            input: None,
        }
    }
}
struct ConcatWorkerInput {
    file_ids: Vec<String>,
}
impl Worker for ConcatWorker {
    fn function_name(&self) -> &str {
        self.func_name.as_str()
    }
    fn function_type(&self) -> FunctionType {
        self.func_type
    }
    fn set_id(&mut self, worker_id: u32) {
        self.worker_id = worker_id;
    }
    fn id(&self) -> u32 {
        self.worker_id
    }
    fn prepare_input(
        &mut self,
        _dynamic_input: Option<String>,
        file_ids: Vec<String>,
    ) -> Result<()> {
        if file_ids.len() < 2 {
            return Err(Error::from(ErrorKind::InvalidInputError));
        }
        self.input = Some(ConcatWorkerInput { file_ids });
        Ok(())
    }
    fn execute(&mut self, context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        let mut result_bytes = Vec::<u8>::new();
        for file_id in input.file_ids.iter() {
            let plaintext = context.read_file(&file_id)?;
            result_bytes.extend_from_slice(&plaintext);
        }
        let result_string = String::from_utf8_lossy(&result_bytes).to_string();
        Ok(result_string)
    }
}

pub struct SwapFileWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<SwapFileWorkerInput>,
}
impl SwapFileWorker {
    pub fn new() -> Self {
        SwapFileWorker {
            worker_id: 0,
            func_name: "swap_file".to_string(),
            func_type: FunctionType::Multiparty,
            input: None,
        }
    }
}
struct SwapFileWorkerInput {
    file_id1: String,
    file_id2: String,
}
impl Worker for SwapFileWorker {
    fn function_name(&self) -> &str {
        self.func_name.as_str()
    }
    fn function_type(&self) -> FunctionType {
        self.func_type
    }
    fn set_id(&mut self, worker_id: u32) {
        self.worker_id = worker_id;
    }
    fn id(&self) -> u32 {
        self.worker_id
    }
    fn prepare_input(
        &mut self,
        _dynamic_input: Option<String>,
        file_ids: Vec<String>,
    ) -> Result<()> {
        if file_ids.len() != 2 {
            return Err(Error::from(ErrorKind::InvalidInputError));
        }

        self.input = Some(SwapFileWorkerInput {
            file_id1: file_ids[0].to_string(),
            file_id2: file_ids[1].to_string(),
        });
        Ok(())
    }
    fn execute(&mut self, context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        let file1 = &input.file_id1;
        let file2 = &input.file_id2;

        let plaintext1 = context.read_file(&file1)?;
        let plaintext2 = context.read_file(&file2)?;
        let mut result = Vec::<u8>::new();
        result.extend_from_slice(&plaintext1);
        result.extend_from_slice(&plaintext2);
        let result_file_id = context.save_file_for_all_participants(&result)?;
        let _file_id = context.save_file_for_file_owner(&plaintext2, file1)?;
        let _file_id = context.save_file_for_file_owner(&plaintext1, file2)?;
        Ok(result_file_id)
    }
}
