use std::collections::HashMap;
use std::prelude::v1::*;

use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{
    ExecutorType, FunctionArguments, Storable, TeaclaveFileCryptoInfo, TeaclaveInputFile,
    TeaclaveOutputFile,
};

const STAGED_TASK_PREFIX: &str = "staged-"; // staged-task-uuid
pub const QUEUE_KEY: &str = "staged-task";

pub type FunctionInputData = HashMap<String, InputDataValue>;
pub type FunctionOutputData = HashMap<String, OutputDataValue>;

#[derive(Debug, Deserialize, Serialize)]
pub struct InputDataValue {
    pub url: Url,
    pub hash: String,
    pub crypto_info: TeaclaveFileCryptoInfo,
}

impl InputDataValue {
    pub fn new(url: Url, hash: impl ToString, crypto_info: TeaclaveFileCryptoInfo) -> Self {
        Self {
            url,
            hash: hash.to_string(),
            crypto_info,
        }
    }

    pub fn from_teaclave_input_file(file: &TeaclaveInputFile) -> Self {
        Self {
            url: file.url.to_owned(),
            hash: file.hash.to_owned(),
            crypto_info: file.crypto_info,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OutputDataValue {
    pub url: Url,
    pub crypto_info: TeaclaveFileCryptoInfo,
}

impl OutputDataValue {
    pub fn new(url: Url, crypto_info: TeaclaveFileCryptoInfo) -> Self {
        Self { url, crypto_info }
    }

    pub fn from_teaclave_output_file(file: &TeaclaveOutputFile) -> Self {
        Self {
            url: file.url.to_owned(),
            crypto_info: file.crypto_info,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct StagedTask {
    pub task_id: Uuid,
    pub function_id: Uuid,
    pub function_name: String,
    pub function_payload: Vec<u8>,
    pub function_arguments: FunctionArguments,
    pub input_data: FunctionInputData,
    pub output_data: FunctionOutputData,
}

impl Storable for StagedTask {
    fn key_prefix() -> &'static str {
        STAGED_TASK_PREFIX
    }

    fn uuid(&self) -> Uuid {
        self.task_id
    }
}

impl StagedTask {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn task_id(self, task_id: Uuid) -> Self {
        Self { task_id, ..self }
    }

    pub fn function_id(self, function_id: Uuid) -> Self {
        Self {
            function_id,
            ..self
        }
    }

    pub fn function_name(self, function_name: impl ToString) -> Self {
        Self {
            function_name: function_name.to_string(),
            ..self
        }
    }

    pub fn function_payload(self, function_payload: Vec<u8>) -> Self {
        Self {
            function_payload,
            ..self
        }
    }

    pub fn function_arguments(self, function_arguments: FunctionArguments) -> Self {
        Self {
            function_arguments,
            ..self
        }
    }

    pub fn input_data(self, input_data: FunctionInputData) -> Self {
        Self { input_data, ..self }
    }

    pub fn output_data(self, output_data: FunctionOutputData) -> Self {
        Self {
            output_data,
            ..self
        }
    }

    pub fn get_queue_key() -> &'static str {
        QUEUE_KEY
    }

    pub fn executor_type(&self) -> ExecutorType {
        if self.function_payload.is_empty() {
            ExecutorType::Native
        } else {
            ExecutorType::Python
        }
    }
}
