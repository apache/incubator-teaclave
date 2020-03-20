use crate::{
    Function, Storable, TeaclaveExecutorSelector, TeaclaveFileCryptoInfo, TeaclaveInputFile,
    TeaclaveOutputFile,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::prelude::v1::*;
use url::Url;
use uuid::Uuid;

const STAGED_TASK_PREFIX: &str = "staged-"; // staged-task-uuid
pub const QUEUE_KEY: &str = "staged-task";

#[derive(Debug, Deserialize, Serialize)]
pub struct InputData {
    pub url: Url,
    pub hash: String,
    pub crypto_info: TeaclaveFileCryptoInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OutputData {
    pub url: Url,
    pub crypto_info: TeaclaveFileCryptoInfo,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct StagedTask {
    pub task_id: Uuid,
    pub function_id: String,
    pub function_name: String,
    pub function_payload: Vec<u8>,
    pub arg_list: HashMap<String, String>,
    pub input_map: HashMap<String, InputData>,
    pub output_map: HashMap<String, OutputData>,
}

impl Storable for StagedTask {
    fn key_prefix() -> &'static str {
        STAGED_TASK_PREFIX
    }

    fn uuid(&self) -> Uuid {
        self.task_id
    }
}

impl InputData {
    pub fn from_input_file(file: TeaclaveInputFile) -> InputData {
        InputData {
            url: file.url,
            hash: file.hash,
            crypto_info: file.crypto_info,
        }
    }
}

impl OutputData {
    pub fn from_output_file(file: TeaclaveOutputFile) -> OutputData {
        OutputData {
            url: file.url,
            crypto_info: file.crypto_info,
        }
    }
}

impl StagedTask {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn task_id(self, task_id: Uuid) -> Self {
        Self { task_id, ..self }
    }

    pub fn function(self, function: &Function) -> Self {
        Self {
            function_id: function.external_id(),
            function_name: function.name.clone(),
            function_payload: function.payload.clone(),
            ..self
        }
    }

    pub fn function_id(self, function_id: impl Into<String>) -> Self {
        Self {
            function_id: function_id.into(),
            ..self
        }
    }

    pub fn function_name(self, function_name: impl Into<String>) -> Self {
        Self {
            function_name: function_name.into(),
            ..self
        }
    }

    pub fn function_payload(self, function_payload: impl Into<Vec<u8>>) -> Self {
        Self {
            function_payload: function_payload.into(),
            ..self
        }
    }

    pub fn args(self, args: HashMap<String, String>) -> Self {
        Self {
            arg_list: args,
            ..self
        }
    }

    pub fn input(self, input: HashMap<String, InputData>) -> Self {
        Self {
            input_map: input,
            ..self
        }
    }

    pub fn output(self, output: HashMap<String, OutputData>) -> Self {
        Self {
            output_map: output,
            ..self
        }
    }

    pub fn get_queue_key() -> &'static str {
        QUEUE_KEY
    }

    pub fn executor_type(&self) -> TeaclaveExecutorSelector {
        if self.function_payload.is_empty() {
            TeaclaveExecutorSelector::Native
        } else {
            TeaclaveExecutorSelector::Python
        }
    }
}
