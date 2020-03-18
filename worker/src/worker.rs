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
    TeaclaveExecutorSelector, TeaclaveFunctionArguments, TeaclaveWorkerFileRegistry,
    TeaclaveWorkerInputFileInfo, TeaclaveWorkerOutputFileInfo, WorkerCapability, WorkerInvocation,
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
                "gbdt_training"     => (TeaclaveExecutorSelector::Native, function::GbdtTraining),
                "gbdt_predition"    => (TeaclaveExecutorSelector::Native, function::GbdtPrediction),
                "echo"              => (TeaclaveExecutorSelector::Native, function::Echo),
                "mesapy"            => (TeaclaveExecutorSelector::Python, function::Mesapy),
            ),
            runtimes: setup_runtimes(),
        }
    }

    pub fn invoke_function(&self, req: WorkerInvocation) -> anyhow::Result<String> {
        let function = self.get_function(req.executor_type, &req.function_name)?;
        let runtime = self.get_runtime(&req.runtime_name, req.input_files, req.output_files)?;
        let unified_args =
            prepare_arguments(req.executor_type, req.function_args, req.function_payload)?;
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
        input_files: TeaclaveWorkerFileRegistry<TeaclaveWorkerInputFileInfo>,
        output_files: TeaclaveWorkerFileRegistry<TeaclaveWorkerOutputFileInfo>,
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
        func_type: TeaclaveExecutorSelector,
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

fn make_function_identifier(func_type: TeaclaveExecutorSelector, func_name: &str) -> String {
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

    runtimes
}

// Native functions (TeaclaveExecutorSelector::Native) are not allowed to have function payload.
// Script engines like Mesapy (TeaclaveExecutorSelector::Python) must have script payload.
// We assume that the script engines would take the script payload and
// script arguments from the wrapped argument.
fn prepare_arguments(
    executor_type: TeaclaveExecutorSelector,
    function_args: TeaclaveFunctionArguments,
    function_payload: String,
) -> anyhow::Result<TeaclaveFunctionArguments> {
    let unified_args = match executor_type {
        TeaclaveExecutorSelector::Native => {
            anyhow::ensure!(
                function_payload.is_empty(),
                "Native function payload should be empty!"
            );
            function_args
        }
        TeaclaveExecutorSelector::Python => {
            anyhow::ensure!(
                !function_payload.is_empty(),
                "Python function payload must not be empty!"
            );
            let mut wrap_args = HashMap::new();
            let req_args = serde_json::to_string(&function_args.args)?;
            wrap_args.insert("py_payload".to_string(), function_payload);
            wrap_args.insert("py_args".to_string(), req_args);
            TeaclaveFunctionArguments { args: wrap_args }
        }
    };

    Ok(unified_args)
}

type FunctionBuilder = Box<dyn Fn() -> Box<dyn TeaclaveFunction + Send + Sync> + Send + Sync>;
type RuntimeBuilder = Box<
    dyn Fn(
            TeaclaveWorkerFileRegistry<TeaclaveWorkerInputFileInfo>,
            TeaclaveWorkerFileRegistry<TeaclaveWorkerOutputFileInfo>,
        ) -> Box<dyn TeaclaveRuntime + Send + Sync>
        + Send
        + Sync,
>;
