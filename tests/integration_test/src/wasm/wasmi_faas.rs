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

use super::wasmi_basic::FaasInterpreterError;
use super::wasmi_basic::{
    answer_convert, spec_to_runtime_value, wabt_runtime_value_to_boundary_value, BoundaryValue,
    RuntimeValue, SgxWasmAction,
};
use wabt::script::{Action, Command, CommandKind, ScriptParser};

fn action_to_sgxaction(action: &Action) -> SgxWasmAction {
    match *action {
        Action::Invoke {
            ref module,
            ref field,
            ref args,
        } => SgxWasmAction::Invoke {
            module: module.as_ref().cloned(),
            field: field.clone(),
            args: args
                .iter()
                .map(wabt_runtime_value_to_boundary_value)
                .collect(),
        },
        Action::Get {
            ref module,
            ref field,
            ..
        } => SgxWasmAction::Get {
            module: module.as_ref().cloned(),
            field: field.clone(),
        },
    }
}

fn command_to_sgxaction(command: &CommandKind) -> SgxWasmAction {
    match command {
        CommandKind::Module { name, module, .. } => SgxWasmAction::LoadModule {
            name: name.as_ref().cloned(),
            module: module.clone().into_vec(),
        },
        CommandKind::AssertReturn { action, .. }
        | CommandKind::AssertReturnCanonicalNan { action }
        | CommandKind::AssertReturnArithmeticNan { action }
        | CommandKind::AssertExhaustion { action, .. }
        | CommandKind::AssertTrap { action, .. }
        | CommandKind::PerformAction(action) => action_to_sgxaction(&action),

        CommandKind::AssertInvalid { module, .. }
        | CommandKind::AssertMalformed { module, .. }
        | CommandKind::AssertUnlinkable { module, .. }
        | CommandKind::AssertUninstantiable { module, .. } => SgxWasmAction::TryLoad {
            module: module.clone().into_vec(),
        },
        CommandKind::Register { name, as_name, .. } => SgxWasmAction::Register {
            name: name.clone(),
            as_name: as_name.to_string(),
        },
    }
}

pub fn match_result(
    commands: Vec<Command>,
    results: Vec<Result<Option<BoundaryValue>, FaasInterpreterError>>,
) -> bool {
    if commands.len() != results.len() {
        return false;
    }
    for (command, result) in commands.iter().zip(results.into_iter()) {
        let result: Result<Option<RuntimeValue>, FaasInterpreterError> = answer_convert(result);
        let line = &command.line;
        match &command.kind {
            CommandKind::Module { .. } => {
                if result.is_err() {
                    println!("failed to load moduel");
                    return false;
                }
            }
            CommandKind::AssertReturn { expected, .. } => {
                match result {
                    Ok(result) => {
                        let spec_expected = expected
                            .iter()
                            .cloned()
                            .map(spec_to_runtime_value)
                            .collect::<Vec<_>>();
                        let actual_result = result.into_iter().collect::<Vec<RuntimeValue>>();
                        for (actual_result, spec_expected) in
                            actual_result.iter().zip(spec_expected.iter())
                        {
                            if actual_result.value_type() != spec_expected.value_type() {
                                return false;
                            }
                            // f32::NAN != f32::NAN
                            match *spec_expected {
                                RuntimeValue::F32(val) if val.is_nan() => match *actual_result {
                                    RuntimeValue::F32(val) => {
                                        if !val.is_nan() {
                                            return false;
                                        }
                                    }
                                    _ => unreachable!(), // checked above that types are same
                                },
                                RuntimeValue::F64(val) if val.is_nan() => match *actual_result {
                                    RuntimeValue::F64(val) => {
                                        if !val.is_nan() {
                                            return false;
                                        }
                                    }
                                    _ => unreachable!(), // checked above that types are same
                                },
                                spec_expected => {
                                    if actual_result != &spec_expected {
                                        return false;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("Expected action to return value, got error: {:?}", e);
                        return false;
                    }
                }
            }
            CommandKind::AssertReturnCanonicalNan { .. }
            | CommandKind::AssertReturnArithmeticNan { .. } => match result {
                Ok(result) => {
                    for actual_result in result.into_iter().collect::<Vec<RuntimeValue>>() {
                        match actual_result {
                            RuntimeValue::F32(val) => {
                                if !val.is_nan() {
                                    println!("Expected nan value, got {:?}", val);
                                    return false;
                                }
                            }
                            RuntimeValue::F64(val) => {
                                if !val.is_nan() {
                                    println!("Expected nan value, got {:?}", val);
                                    return false;
                                }
                            }
                            val => {
                                println!("Expected action to return float value, got {:?}", val);
                                return false;
                            }
                        }
                    }
                    //println!("assert_return_nan at line {} - success", line);
                }
                Err(e) => {
                    println!("Expected action to return value, got error: {:?}", e);
                    return false;
                }
            },
            CommandKind::AssertExhaustion { .. } => {
                if let Ok(result) = result {
                    println!("Expected exhaustion, got result: {:?}", result);
                    return false;
                }
            }
            CommandKind::AssertTrap { .. } => {
                if let Ok(result) = result {
                    println!(
                        "Expected action to result in a trap, got result: {:?}",
                        result
                    );
                    return false;
                }
            }
            CommandKind::AssertInvalid { .. }
            | CommandKind::AssertMalformed { .. }
            | CommandKind::AssertUnlinkable { .. } => {
                if result.is_ok() {
                    println!("Expected invalid module definition, got some module!");
                    return false;
                }
            }
            CommandKind::AssertUninstantiable { .. } => {
                if result.is_ok() {
                    println!("Expected error running start function at line {}", line);
                    return false;
                }
            }
            CommandKind::Register { .. } => {
                if let Err(e) = result {
                    println!("No such module - ({:?})", e);
                    return false;
                }
            }
            CommandKind::PerformAction(_) => {
                if let Err(e) = result {
                    println!("Failed to invoke action {:?}", e);
                    return false;
                }
            }
        }
    }
    true
}

pub fn parse_a_wast(wast_file: &str) -> Result<Vec<Command>, String> {
    let wast_content: Vec<u8> = std::fs::read(wast_file).unwrap();
    let path = std::path::Path::new(wast_file);
    let fnme = path.file_name().unwrap().to_str().unwrap();
    let mut parser = ScriptParser::from_source_and_name(&wast_content, fnme).unwrap();
    let mut commands: Vec<Command> = Vec::new();
    while let Some(command) = match parser.next() {
        Ok(x) => x,
        _ => {
            return Err("Error parsing test input".to_string());
        }
    } {
        commands.push(command);
    }
    Ok(commands)
}

pub fn get_sgx_action(commands: &[Command]) -> Vec<SgxWasmAction> {
    let ret: Vec<SgxWasmAction> = commands
        .iter()
        .map(|command| command_to_sgxaction(&command.kind))
        .collect();
    ret
}
