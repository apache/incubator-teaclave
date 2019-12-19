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

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use super::sgxwasm;
use super::sgxwasm::{boundary_value_to_runtime_value, result_covert, SpecDriver};
use super::sgxwasm::{BoundaryValue, InterpreterError};
use std::format;
use wasmi::{ImportsBuilder, Module, ModuleInstance, RuntimeValue};

pub fn sgxwasm_init() -> SpecDriver {
    SpecDriver::new()
}

pub fn sgxwasm_run_action(
    spec_driver: &mut SpecDriver,
    action_req: sgxwasm::SgxWasmAction,
) -> Result<Option<BoundaryValue>, InterpreterError> {
    let response;

    match action_req {
        sgxwasm::SgxWasmAction::Invoke {
            module,
            field,
            args,
        } => {
            let args = args
                .into_iter()
                .map(boundary_value_to_runtime_value)
                .collect::<Vec<RuntimeValue>>();
            let r = wasm_invoke(spec_driver, module, field, args);
            response = result_covert(r);
        }
        sgxwasm::SgxWasmAction::Get { module, field } => {
            let r = wasm_get(spec_driver, module, field);
            response = result_covert(r);
        }
        sgxwasm::SgxWasmAction::LoadModule { name, module } => {
            let r = wasm_load_module(spec_driver, name, module);
            response = r.map(|_| Option::<BoundaryValue>::None);
        }
        sgxwasm::SgxWasmAction::TryLoad { module } => {
            let r = wasm_try_load(spec_driver, module);
            response = r.map(|_| Option::<BoundaryValue>::None);
        }
        sgxwasm::SgxWasmAction::Register { name, as_name } => {
            let r = wasm_register(spec_driver, &name, as_name);
            response = r.map(|_| Option::<BoundaryValue>::None);
        }
    }

    response
}

fn wasm_invoke(
    program: &mut SpecDriver,
    module: Option<String>,
    field: String,
    args: Vec<RuntimeValue>,
) -> Result<Option<RuntimeValue>, InterpreterError> {
    let module = program.module_or_last(module.as_ref().map(|x| x.as_ref()))?;
    module.invoke_export(&field, &args, program.spec_module())
}

fn wasm_get(
    program: &mut SpecDriver,
    module: Option<String>,
    field: String,
) -> Result<Option<RuntimeValue>, InterpreterError> {
    let module = match module {
        None => program.module_or_last(None)?,
        Some(str) => program.module_or_last(Some(&str))?,
    };

    let global = module
        .export_by_name(&field)
        .ok_or_else(|| {
            InterpreterError::Global(format!("Expected to have export with name {}", field))
        })?
        .as_global()
        .cloned()
        .ok_or_else(|| {
            InterpreterError::Global(format!("Expected export {} to be a global", field))
        })?;
    Ok(Some(global.get()))
}

fn try_load_module(wasm: &[u8]) -> Result<Module, InterpreterError> {
    wasmi::Module::from_buffer(wasm)
        .map_err(|e| InterpreterError::Instantiation(format!("Module::from_buffer error {:?}", e)))
}

fn wasm_try_load(spec_driver: &mut SpecDriver, wasm: Vec<u8>) -> Result<(), InterpreterError> {
    let module = try_load_module(&wasm[..])?;
    let instance = ModuleInstance::new(&module, &ImportsBuilder::default())?;
    instance
        .run_start(spec_driver.spec_module())
        .map_err(|trap| {
            InterpreterError::Instantiation(format!(
                "ModuleInstance::run_start error on {:?}",
                trap
            ))
        })?;
    Ok(())
}

fn wasm_load_module(
    spec_driver: &mut SpecDriver,
    name: Option<String>,
    module: Vec<u8>,
) -> Result<(), InterpreterError> {
    let module = try_load_module(&module[..])?;
    let instance = ModuleInstance::new(&module, spec_driver)
        .map_err(|e| {
            InterpreterError::Instantiation(format!("ModuleInstance::new error on {:?}", e))
        })?
        .run_start(spec_driver.spec_module())
        .map_err(|trap| {
            InterpreterError::Instantiation(format!(
                "ModuleInstance::run_start error on {:?}",
                trap
            ))
        })?;

    spec_driver.add_module(name, instance);

    Ok(())
}

fn wasm_register(
    spec_driver: &mut SpecDriver,
    name: &Option<String>,
    as_name: String,
) -> Result<(), InterpreterError> {
    spec_driver.register(name, as_name)
}
