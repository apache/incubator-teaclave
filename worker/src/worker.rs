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

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use std::collections::HashMap;
use std::format;

use anyhow;
use serde_json;

use teaclave_types::{
    hashmap, ExecutorType, FunctionArguments, StagedFiles, StagedFunction, StagedInputFile,
    StagedOutputFile, WorkerCapability,
};

use crate::function::{self, TeaclaveFunction};
use crate::runtime::{self, TeaclaveRuntime};

macro_rules! register_functions{
    ($($name: expr => ($executor: expr, $fn_type: ty),)*) => {{
        let mut functions: HashMap<String, FunctionBuilder> = HashMap::new();
        $(
            functions.insert(
                make_function_identifier($executor, $name),
                Box::new(|| Box::new(<$fn_type>::default())),
            );
        )*
        functions
    }}
}

pub struct Worker {
    runtimes: HashMap<String, RuntimeBuilder>,
    functions: HashMap<String, FunctionBuilder>,
}

impl Worker {
    pub fn default() -> Worker {
        Worker {
            functions: register_functions!(
                "gbdt_training"     => (ExecutorType::Native, function::GbdtTraining),
                "gbdt_prediction"    => (ExecutorType::Native, function::GbdtPrediction),
                "echo"              => (ExecutorType::Native, function::Echo),
                "mesapy"            => (ExecutorType::Python, function::Mesapy),
            ),
            runtimes: setup_runtimes(),
        }
    }

    pub fn invoke_function(&self, staged_function: StagedFunction) -> anyhow::Result<String> {
        let function = self.get_function(staged_function.executor_type, &staged_function.name)?;
        let runtime = self.get_runtime(
            &staged_function.runtime_name,
            staged_function.input_files,
            staged_function.output_files,
        )?;
        let unified_args = prepare_arguments(
            staged_function.executor_type,
            staged_function.arguments,
            staged_function.payload,
        )?;
        function.execute(runtime, unified_args)
    }

    pub fn get_capability(&self) -> WorkerCapability {
        WorkerCapability {
            runtimes: self.runtimes.keys().cloned().collect(),
            functions: self.functions.keys().cloned().collect(),
        }
    }

    fn get_runtime(
        &self,
        name: &str,
        input_files: StagedFiles<StagedInputFile>,
        output_files: StagedFiles<StagedOutputFile>,
    ) -> anyhow::Result<Box<dyn TeaclaveRuntime + Send + Sync>> {
        let build_runtime = self
            .runtimes
            .get(name)
            .ok_or_else(|| anyhow::anyhow!(format!("Runtime {} not available.", name)))?;

        let runtime = build_runtime(input_files, output_files);
        Ok(runtime)
    }

    fn get_function(
        &self,
        func_type: ExecutorType,
        func_name: &str,
    ) -> anyhow::Result<Box<dyn TeaclaveFunction + Send + Sync>> {
        let identifier = make_function_identifier(func_type, func_name);
        let build_function = self
            .functions
            .get(&identifier)
            .ok_or_else(|| anyhow::anyhow!(format!("function not available: {}", identifier)))?;

        let function = build_function();
        Ok(function)
    }
}

fn make_function_identifier(func_type: ExecutorType, func_name: &str) -> String {
    let type_str = func_type.to_string();
    format!("{}-{}", type_str, func_name)
}

fn setup_runtimes() -> HashMap<String, RuntimeBuilder> {
    let mut runtimes: HashMap<String, RuntimeBuilder> = HashMap::new();
    runtimes.insert(
        "default".to_string(),
        Box::new(|input_files, output_files| {
            Box::new(runtime::DefaultRuntime::new(input_files, output_files))
        }),
    );
    #[cfg(test_mode)]
    runtimes.insert(
        "raw-io".to_string(),
        Box::new(|input_files, output_files| {
            Box::new(runtime::RawIoRuntime::new(input_files, output_files))
        }),
    );

    runtimes
}

// Native functions (ExecutorType::Native) are not allowed to have function payload.
// Script engines like Mesapy (ExecutorType::Python) must have script payload.
// We assume that the script engines would take the script payload and
// script arguments from the wrapped argument.
fn prepare_arguments(
    executor_type: ExecutorType,
    function_arguments: FunctionArguments,
    function_payload: String,
) -> anyhow::Result<FunctionArguments> {
    let unified_args = match executor_type {
        ExecutorType::Native => {
            anyhow::ensure!(
                function_payload.is_empty(),
                "Native function payload should be empty!"
            );
            function_arguments
        }
        ExecutorType::Python => {
            anyhow::ensure!(
                !function_payload.is_empty(),
                "Python function payload must not be empty!"
            );
            let req_args = serde_json::to_string(&function_arguments)?;
            let wrap_args = hashmap!(
                "py_payload" => function_payload,
                "py_args" => req_args,
            );
            FunctionArguments::new(wrap_args)
        }
    };

    Ok(unified_args)
}

type FunctionBuilder = Box<dyn Fn() -> Box<dyn TeaclaveFunction + Send + Sync> + Send + Sync>;
type RuntimeBuilder = Box<
    dyn Fn(
            StagedFiles<StagedInputFile>,
            StagedFiles<StagedOutputFile>,
        ) -> Box<dyn TeaclaveRuntime + Send + Sync>
        + Send
        + Sync,
>;
