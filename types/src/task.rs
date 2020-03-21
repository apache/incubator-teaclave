use crate::FunctionArguments;
use crate::Storable;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::prelude::v1::*;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DataOwnerList {
    pub user_id_list: HashSet<String>,
}

#[derive(Debug, Deserialize, Serialize, std::cmp::PartialEq)]
pub enum TaskStatus {
    Created,
    Ready,
    Approved,
    Running,
    Failed,
    Finished,
}

const TASK_PREFIX: &str = "task-";

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub task_id: Uuid,
    pub creator: String,
    pub function_id: String,
    pub function_owner: String,
    pub function_arguments: FunctionArguments,
    pub input_data_owner_list: HashMap<String, DataOwnerList>,
    pub output_data_owner_list: HashMap<String, DataOwnerList>,
    pub participants: HashSet<String>,
    pub approved_user_list: HashSet<String>,
    pub input_map: HashMap<String, String>,
    pub output_map: HashMap<String, String>,
    pub return_value: Option<Vec<u8>>,
    pub output_file_hash: HashMap<String, String>,
    pub status: TaskStatus,
}

impl Storable for Task {
    fn key_prefix() -> &'static str {
        TASK_PREFIX
    }

    fn uuid(&self) -> Uuid {
        self.task_id
    }
}
