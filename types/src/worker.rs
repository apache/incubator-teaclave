use std::collections::HashMap;
use std::collections::HashSet;
use std::prelude::v1::*;

use anyhow;

#[derive(Debug, Copy, Clone)]
pub enum ExecutorType {
    Native,
    Python,
}

impl std::default::Default for ExecutorType {
    fn default() -> Self {
        ExecutorType::Native
    }
}

impl std::convert::TryFrom<&str> for ExecutorType {
    type Error = anyhow::Error;

    fn try_from(selector: &str) -> anyhow::Result<Self> {
        let sel = match selector {
            "python" => ExecutorType::Python,
            "native" => ExecutorType::Native,
            _ => anyhow::bail!("Invalid executor selector: {}", selector),
        };
        Ok(sel)
    }
}

impl std::fmt::Display for ExecutorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExecutorType::Native => write!(f, "native"),
            ExecutorType::Python => write!(f, "python"),
        }
    }
}

#[derive(Debug)]
pub struct WorkerCapability {
    pub runtimes: HashSet<String>,
    pub functions: HashSet<String>,
}

#[derive(Default)]
pub struct ExecutionResult {
    pub return_value: Vec<u8>,
    pub output_file_hash: HashMap<String, String>,
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    //use crate::unit_tests;
    //use crate::unittest::*;

    pub fn run_tests() -> bool {
        //unit_tests!()
        true
    }
}
