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

use teaclave_types::{
    hashmap, Executor, ExecutorType, FunctionArguments, StagedFiles, StagedFunction,
    WorkerCapability,
};

use teaclave_function as function;
use teaclave_runtime as runtime;
use teaclave_types::{TeaclaveFunction, TeaclaveRuntime};

macro_rules! register_functions{
    ($(($executor_type: expr, $executor_name: expr) => $fn_type: ty,)*) => {{
        let mut functions: HashMap<(ExecutorType, Executor), FunctionBuilder> = HashMap::new();
        $(
            functions.insert(
                ($executor_type, $executor_name),
                Box::new(|| Box::new(<$fn_type>::default())),
            );
        )*
        functions
    }}
}

pub struct Worker {
    runtimes: HashMap<String, RuntimeBuilder>,
    functions: HashMap<(ExecutorType, Executor), FunctionBuilder>,
}

impl Worker {
    pub fn default() -> Worker {
        Worker {
            functions: register_functions!(
                (ExecutorType::Python, Executor::MesaPy) => function::Mesapy,
                (ExecutorType::Native, Executor::Echo) => function::Echo,
                (ExecutorType::Native, Executor::GbdtTraining) => function::GbdtTraining,
                (ExecutorType::Native, Executor::GbdtPrediction) => function::GbdtPrediction,
                (ExecutorType::Native, Executor::LogitRegTraining) => function::LogitRegTraining,
                (ExecutorType::Native, Executor::LogitRegPrediction) => function::LogitRegPrediction,
            ),
            runtimes: setup_runtimes(),
        }
    }

    pub fn invoke_function(&self, staged_function: StagedFunction) -> anyhow::Result<String> {
        let function =
            self.get_function(staged_function.executor_type, staged_function.executor)?;
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
            functions: self
                .functions
                .keys()
                .cloned()
                .map(|(exec_type, exec_name)| make_function_identifier(exec_type, exec_name))
                .collect(),
        }
    }

    fn get_runtime(
        &self,
        name: &str,
        input_files: StagedFiles,
        output_files: StagedFiles,
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
        exec_type: ExecutorType,
        exec_name: Executor,
    ) -> anyhow::Result<Box<dyn TeaclaveFunction + Send + Sync>> {
        let identifier = (exec_type, exec_name);
        let build_function = self
            .functions
            .get(&identifier)
            .ok_or_else(|| anyhow::anyhow!(format!("function not available: {:?}", identifier)))?;

        let function = build_function();
        Ok(function)
    }
}

fn make_function_identifier(exec_type: ExecutorType, exec_name: Executor) -> String {
    format!("{}-{}", exec_type, exec_name)
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
type RuntimeBuilder =
    Box<dyn Fn(StagedFiles, StagedFiles) -> Box<dyn TeaclaveRuntime + Send + Sync> + Send + Sync>;
