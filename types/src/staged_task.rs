use crate::{Function, Storable, TeaclaveFileCryptoInfo, TeaclaveInputFile, TeaclaveOutputFile};
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

#[derive(Debug, Deserialize, Serialize)]
pub struct StagedTask {
    pub task_id: Uuid,
    pub function_id: String,
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
    pub fn new(
        task_id: Uuid,
        function: Function,
        arg_list: HashMap<String, String>,
        input_map: HashMap<String, InputData>,
        output_map: HashMap<String, OutputData>,
    ) -> Self {
        Self {
            task_id: task_id.to_owned(),
            function_id: function.external_id(),
            function_payload: function.payload,
            arg_list,
            input_map,
            output_map,
        }
    }

    pub fn get_queue_key() -> &'static str {
        QUEUE_KEY
    }
}
