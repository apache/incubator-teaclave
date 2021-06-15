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

use teaclave_types::{Executor, ExecutorType, StagedFiles, StagedFunction};

use teaclave_executor::{BuiltinFunctionExecutor, MesaPy, WAMicroRuntime};
use teaclave_runtime::DefaultRuntime;
use teaclave_types::{TeaclaveExecutor, TeaclaveRuntime};

type BoxedTeaclaveExecutor = Box<dyn TeaclaveExecutor + Send + Sync>;
type BoxedTeaclaveRuntime = Box<dyn TeaclaveRuntime + Send + Sync>;
type ExecutorBuilder = fn() -> BoxedTeaclaveExecutor;
type RuntimeBuilder = fn(StagedFiles, StagedFiles) -> BoxedTeaclaveRuntime;

pub struct Worker {
    runtimes: HashMap<String, RuntimeBuilder>,
    executors: HashMap<(ExecutorType, Executor), ExecutorBuilder>,
}

impl Default for Worker {
    fn default() -> Self {
        let mut worker = Worker::new();

        // Register supported runtimes
        worker.register_runtime("default", |input, output| {
            Box::new(DefaultRuntime::new(input, output))
        });

        #[cfg(test_mode)]
        worker.register_runtime("raw-io", |input, output| {
            Box::new(teaclave_runtime::RawIoRuntime::new(input, output))
        });

        // Register supported executors
        worker.register_executor((ExecutorType::Python, Executor::MesaPy), || {
            Box::new(MesaPy::default())
        });
        worker.register_executor((ExecutorType::Builtin, Executor::Builtin), || {
            Box::new(BuiltinFunctionExecutor::default())
        });
        worker.register_executor(
            (ExecutorType::WAMicroRuntime, Executor::WAMicroRuntime),
            || Box::new(WAMicroRuntime::default()),
        );

        worker
    }
}

impl Worker {
    pub fn new() -> Self {
        Self {
            runtimes: HashMap::new(),
            executors: HashMap::new(),
        }
    }

    pub fn register_runtime(&mut self, name: impl ToString, builder: RuntimeBuilder) {
        self.runtimes.insert(name.to_string(), builder);
    }

    pub fn register_executor(&mut self, key: (ExecutorType, Executor), builder: ExecutorBuilder) {
        self.executors.insert(key, builder);
    }

    pub fn invoke_function(&self, function: StagedFunction) -> anyhow::Result<String> {
        let executor = self.get_executor(function.executor_type, function.executor)?;
        let runtime = self.get_runtime(
            &function.runtime_name,
            function.input_files,
            function.output_files,
        )?;
        executor.execute(function.name, function.arguments, function.payload, runtime)
    }

    fn get_runtime(
        &self,
        name: &str,
        input_files: StagedFiles,
        output_files: StagedFiles,
    ) -> anyhow::Result<BoxedTeaclaveRuntime> {
        let build_runtime = self
            .runtimes
            .get(name)
            .ok_or_else(|| anyhow::anyhow!(format!("Runtime {} not available.", name)))?;

        let runtime = build_runtime(input_files, output_files);
        Ok(runtime)
    }

    fn get_executor(
        &self,
        exec_type: ExecutorType,
        exec_name: Executor,
    ) -> anyhow::Result<BoxedTeaclaveExecutor> {
        let identifier = (exec_type, exec_name);
        let build_executor = self
            .executors
            .get(&identifier)
            .ok_or_else(|| anyhow::anyhow!(format!("function not available: {:?}", identifier)))?;

        let executor = build_executor();

        Ok(executor)
    }
}
