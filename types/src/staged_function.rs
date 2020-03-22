use crate::{ExecutorType, StagedFiles, StagedInputFile, StagedOutputFile};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::prelude::v1::*;
use std::str::FromStr;

use anyhow::{Context, Result};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArgumentValue {
    inner: String,
}

impl From<String> for ArgumentValue {
    fn from(value: String) -> Self {
        ArgumentValue::new(value)
    }
}

impl From<&str> for ArgumentValue {
    fn from(value: &str) -> Self {
        ArgumentValue::new(value.into())
    }
}

impl From<&String> for ArgumentValue {
    fn from(value: &String) -> Self {
        ArgumentValue::new(value.into())
    }
}

impl From<ArgumentValue> for String {
    fn from(value: ArgumentValue) -> Self {
        value.as_str().to_owned()
    }
}

impl ArgumentValue {
    pub fn new(value: String) -> Self {
        Self { inner: value }
    }

    pub fn inner(&self) -> &String {
        &self.inner
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn as_usize(&self) -> Result<usize> {
        usize::from_str(&self.inner).with_context(|| format!("cannot parse {}", self.inner))
    }

    pub fn as_u32(&self) -> Result<u32> {
        u32::from_str(&self.inner).with_context(|| format!("cannot parse {}", self.inner))
    }

    pub fn as_f32(&self) -> Result<f32> {
        f32::from_str(&self.inner).with_context(|| format!("cannot parse {}", self.inner))
    }

    pub fn as_f64(&self) -> Result<f64> {
        f64::from_str(&self.inner).with_context(|| format!("cannot parse {}", self.inner))
    }

    pub fn as_u8(&self) -> Result<u8> {
        u8::from_str(&self.inner).with_context(|| format!("cannot parse {}", self.inner))
    }
}

impl std::fmt::Display for ArgumentValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct FunctionArguments {
    #[serde(flatten)]
    pub inner: HashMap<String, ArgumentValue>,
}

impl<S: core::default::Default + std::hash::BuildHasher> From<FunctionArguments>
    for HashMap<String, String, S>
{
    fn from(arguments: FunctionArguments) -> Self {
        arguments
            .inner()
            .iter()
            .map(|(k, v)| (k.to_owned(), v.as_str().to_owned()))
            .collect()
    }
}

impl From<HashMap<String, String>> for FunctionArguments {
    fn from(map: HashMap<String, String>) -> Self {
        let inner = map.iter().fold(HashMap::new(), |mut acc, (k, v)| {
            acc.insert(k.into(), v.into());
            acc
        });

        Self { inner }
    }
}

impl FunctionArguments {
    pub fn new(map: HashMap<String, ArgumentValue>) -> Self {
        Self { inner: map }
    }

    pub fn inner(&self) -> &HashMap<String, ArgumentValue> {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut HashMap<String, ArgumentValue> {
        &mut self.inner
    }

    pub fn get(&self, key: &str) -> anyhow::Result<&ArgumentValue> {
        self.inner
            .get(key)
            .with_context(|| format!("key not found: {}", key))
    }

    pub fn into_vec(self) -> Vec<String> {
        let mut vector = Vec::new();

        self.inner.into_iter().for_each(|(k, v)| {
            vector.push(k);
            vector.push(v.to_string());
        });

        vector
    }
}

#[derive(Debug, Default)]
pub struct StagedFunction {
    pub name: String,
    pub payload: String,
    pub arguments: FunctionArguments,
    pub input_files: StagedFiles<StagedInputFile>,
    pub output_files: StagedFiles<StagedOutputFile>,
    pub runtime_name: String,
    pub executor_type: ExecutorType,
}

impl StagedFunction {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(self, name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            ..self
        }
    }

    pub fn payload(self, payload: impl ToString) -> Self {
        Self {
            payload: payload.to_string(),
            ..self
        }
    }

    pub fn arguments(self, arguments: FunctionArguments) -> Self {
        Self { arguments, ..self }
    }

    pub fn input_files(self, input_files: StagedFiles<StagedInputFile>) -> Self {
        Self {
            input_files,
            ..self
        }
    }

    pub fn output_files(self, output_files: StagedFiles<StagedOutputFile>) -> Self {
        Self {
            output_files,
            ..self
        }
    }

    pub fn runtime_name(self, runtime_name: impl ToString) -> Self {
        Self {
            runtime_name: runtime_name.to_string(),
            ..self
        }
    }

    pub fn executor_type(self, executor_type: ExecutorType) -> Self {
        Self {
            executor_type,
            ..self
        }
    }
}
