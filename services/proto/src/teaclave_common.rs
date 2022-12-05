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

use crate::teaclave_common_proto as proto;
use anyhow::{bail, Error, Result};
use std::convert::TryInto;
use teaclave_crypto::TeaclaveFile128Key;
use teaclave_types::{FileCrypto, TaskFailure, TaskOutputs, TaskResult, TaskStatus};

#[derive(Debug)]
pub struct UserCredential {
    pub id: std::string::String,
    pub token: std::string::String,
}

impl UserCredential {
    pub fn new(id: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            token: token.into(),
        }
    }
}

impl std::convert::TryFrom<proto::UserCredential> for UserCredential {
    type Error = Error;

    fn try_from(proto: proto::UserCredential) -> Result<Self> {
        let ret = Self {
            id: proto.id,
            token: proto.token,
        };

        Ok(ret)
    }
}

impl From<UserCredential> for proto::UserCredential {
    fn from(request: UserCredential) -> Self {
        Self {
            id: request.id,
            token: request.token,
        }
    }
}

impl std::convert::TryFrom<proto::FileCryptoInfo> for FileCrypto {
    type Error = Error;
    fn try_from(proto: proto::FileCryptoInfo) -> Result<Self> {
        FileCrypto::new(&proto.schema, &proto.key, &proto.iv)
    }
}

impl std::convert::TryFrom<proto::FileCryptoInfo> for TeaclaveFile128Key {
    type Error = Error;
    fn try_from(proto: proto::FileCryptoInfo) -> Result<Self> {
        let file_crypto = FileCrypto::new(&proto.schema, &proto.key, &proto.iv)?;
        let crypto = match file_crypto {
            FileCrypto::TeaclaveFile128(info) => info,
            _ => anyhow::bail!("FileCryptoInfo not supported"),
        };
        Ok(crypto)
    }
}

impl std::convert::From<FileCrypto> for proto::FileCryptoInfo {
    fn from(crypto: FileCrypto) -> Self {
        let (key, iv) = crypto.key_iv();
        proto::FileCryptoInfo {
            schema: crypto.schema().to_owned(),
            key,
            iv,
        }
    }
}

impl std::convert::From<TeaclaveFile128Key> for proto::FileCryptoInfo {
    fn from(crypto: TeaclaveFile128Key) -> Self {
        let crypto = FileCrypto::TeaclaveFile128(crypto);
        let (key, iv) = crypto.key_iv();
        proto::FileCryptoInfo {
            schema: crypto.schema().to_owned(),
            key,
            iv,
        }
    }
}

pub fn i32_to_task_status(status: i32) -> Result<TaskStatus> {
    let ret = match proto::TaskStatus::from_i32(status) {
        Some(proto::TaskStatus::Created) => TaskStatus::Created,
        Some(proto::TaskStatus::DataAssigned) => TaskStatus::DataAssigned,
        Some(proto::TaskStatus::Approved) => TaskStatus::Approved,
        Some(proto::TaskStatus::Staged) => TaskStatus::Staged,
        Some(proto::TaskStatus::Running) => TaskStatus::Running,
        Some(proto::TaskStatus::Finished) => TaskStatus::Finished,
        Some(proto::TaskStatus::Failed) => TaskStatus::Failed,
        Some(proto::TaskStatus::Canceled) => TaskStatus::Canceled,
        None => bail!("invalid task status"),
    };
    Ok(ret)
}

pub fn i32_from_task_status(status: TaskStatus) -> i32 {
    match status {
        TaskStatus::Created => proto::TaskStatus::Created as i32,
        TaskStatus::DataAssigned => proto::TaskStatus::DataAssigned as i32,
        TaskStatus::Approved => proto::TaskStatus::Approved as i32,
        TaskStatus::Staged => proto::TaskStatus::Staged as i32,
        TaskStatus::Running => proto::TaskStatus::Running as i32,
        TaskStatus::Finished => proto::TaskStatus::Finished as i32,
        TaskStatus::Failed => proto::TaskStatus::Failed as i32,
        TaskStatus::Canceled => proto::TaskStatus::Canceled as i32,
    }
}

impl std::convert::TryFrom<proto::TaskOutputs> for TaskOutputs {
    type Error = Error;
    fn try_from(proto: proto::TaskOutputs) -> Result<Self> {
        let ret = TaskOutputs {
            return_value: proto.return_value,
            tags_map: proto.tags_map.try_into()?,
        };
        Ok(ret)
    }
}
impl std::convert::From<TaskOutputs> for proto::TaskOutputs {
    fn from(outputs: TaskOutputs) -> Self {
        proto::TaskOutputs {
            return_value: outputs.return_value,
            tags_map: outputs.tags_map.into(),
        }
    }
}

impl std::convert::TryFrom<proto::TaskFailure> for TaskFailure {
    type Error = Error;
    fn try_from(proto: proto::TaskFailure) -> Result<Self> {
        let ret = TaskFailure {
            reason: proto.reason,
        };
        Ok(ret)
    }
}
impl std::convert::From<TaskFailure> for proto::TaskFailure {
    fn from(outputs: TaskFailure) -> Self {
        proto::TaskFailure {
            reason: outputs.reason,
        }
    }
}

impl std::convert::TryFrom<proto::TaskResult> for TaskResult {
    type Error = Error;
    fn try_from(proto: proto::TaskResult) -> Result<Self> {
        let task_result = match proto.result {
            Some(proto_result) => match proto_result {
                proto::task_result::Result::Ok(task_outputs) => {
                    let outputs_info = task_outputs.try_into()?;
                    TaskResult::Ok(outputs_info)
                }
                proto::task_result::Result::Err(task_failure) => {
                    let failure_info = task_failure.try_into()?;
                    TaskResult::Err(failure_info)
                }
            },
            None => TaskResult::NotReady,
        };
        Ok(task_result)
    }
}

impl std::convert::From<TaskResult> for proto::TaskResult {
    fn from(result: TaskResult) -> Self {
        let opt_result = match result {
            TaskResult::Ok(outputs) => Some(proto::task_result::Result::Ok(outputs.into())),
            TaskResult::Err(failure) => Some(proto::task_result::Result::Err(failure.into())),
            TaskResult::NotReady => None,
        };

        proto::TaskResult { result: opt_result }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExecutorStatus {
    Idle,
    Executing,
}

impl std::convert::TryFrom<proto::ExecutorStatus> for ExecutorStatus {
    type Error = Error;
    fn try_from(status: proto::ExecutorStatus) -> Result<Self> {
        match status {
            proto::ExecutorStatus::Idle => Ok(ExecutorStatus::Idle),
            proto::ExecutorStatus::Executing => Ok(ExecutorStatus::Executing),
        }
    }
}

impl std::convert::TryFrom<i32> for ExecutorStatus {
    type Error = Error;
    fn try_from(status: i32) -> Result<Self> {
        match proto::ExecutorStatus::from_i32(status) {
            Some(proto::ExecutorStatus::Idle) => Ok(ExecutorStatus::Idle),
            Some(proto::ExecutorStatus::Executing) => Ok(ExecutorStatus::Executing),
            _ => bail!("invalid executor status"),
        }
    }
}

impl std::convert::From<ExecutorStatus> for i32 {
    fn from(status: ExecutorStatus) -> Self {
        match status {
            ExecutorStatus::Idle => proto::ExecutorStatus::Idle as i32,
            ExecutorStatus::Executing => proto::ExecutorStatus::Executing as i32,
        }
    }
}

impl std::convert::From<ExecutorStatus> for proto::ExecutorStatus {
    fn from(status: ExecutorStatus) -> Self {
        match status {
            ExecutorStatus::Idle => proto::ExecutorStatus::Idle,
            ExecutorStatus::Executing => proto::ExecutorStatus::Executing,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutorCommand {
    NoAction,
    Stop,
    NewTask,
}

impl Default for ExecutorCommand {
    fn default() -> Self {
        ExecutorCommand::NoAction
    }
}

impl std::convert::TryFrom<proto::ExecutorCommand> for ExecutorCommand {
    type Error = Error;
    fn try_from(command: proto::ExecutorCommand) -> Result<Self> {
        match command {
            proto::ExecutorCommand::NoAction => Ok(ExecutorCommand::NoAction),
            proto::ExecutorCommand::Stop => Ok(ExecutorCommand::Stop),
            proto::ExecutorCommand::NewTask => Ok(ExecutorCommand::NewTask),
        }
    }
}

impl std::convert::From<ExecutorCommand> for proto::ExecutorCommand {
    fn from(command: ExecutorCommand) -> Self {
        match command {
            ExecutorCommand::NoAction => proto::ExecutorCommand::NoAction,
            ExecutorCommand::Stop => proto::ExecutorCommand::Stop,
            ExecutorCommand::NewTask => proto::ExecutorCommand::NewTask,
        }
    }
}

impl std::convert::TryFrom<i32> for ExecutorCommand {
    type Error = Error;
    fn try_from(command: i32) -> Result<Self> {
        match proto::ExecutorCommand::from_i32(command) {
            Some(proto::ExecutorCommand::NoAction) => Ok(ExecutorCommand::NoAction),
            Some(proto::ExecutorCommand::Stop) => Ok(ExecutorCommand::Stop),
            Some(proto::ExecutorCommand::NewTask) => Ok(ExecutorCommand::NewTask),
            _ => bail!("invalid executor status"),
        }
    }
}

impl std::convert::From<ExecutorCommand> for i32 {
    fn from(command: ExecutorCommand) -> Self {
        match command {
            ExecutorCommand::NoAction => proto::ExecutorCommand::NoAction as i32,
            ExecutorCommand::Stop => proto::ExecutorCommand::Stop as i32,
            ExecutorCommand::NewTask => proto::ExecutorCommand::NewTask as i32,
        }
    }
}
