// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use nan_preserving_float::{F32, F64};
use serde_derive::*;
use std::{error, fmt};

#[derive(Deserialize)]
pub struct FaasInterpreterError;
impl fmt::Debug for FaasInterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "faas interperter error")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trap {
    kind: TrapKind,
}

impl Trap {
    /// Create new trap.
    pub fn new(kind: TrapKind) -> Trap {
        Trap { kind }
    }

    /*
    /// Returns kind of this trap.
    pub fn kind(&self) -> &TrapKind {
        &self.kind
    }*/
}

/// Error type which can thrown by wasm code or by host environment.
///
/// See [`Trap`] for details.
///
/// [`Trap`]: struct.Trap.html
#[derive(Debug, Serialize, Deserialize)]
pub enum TrapKind {
    /// Wasm code executed `unreachable` opcode.
    ///
    /// `unreachable` is a special opcode which always traps upon execution.
    /// This opcode have a similar purpose as `ud2` in x86.
    Unreachable,

    /// Attempt to load or store at the address which
    /// lies outside of bounds of the memory.
    ///
    /// Since addresses are interpreted as unsigned integers, out of bounds access
    /// can't happen with negative addresses (i.e. they will always wrap).
    MemoryAccessOutOfBounds,

    /// Attempt to access table element at index which
    /// lies outside of bounds.
    ///
    /// This typically can happen when `call_indirect` is executed
    /// with index that lies out of bounds.
    ///
    /// Since indexes are interpreted as unsinged integers, out of bounds access
    /// can't happen with negative indexes (i.e. they will always wrap).
    TableAccessOutOfBounds,

    /// Attempt to access table element which is uninitialized (i.e. `None`).
    ///
    /// This typically can happen when `call_indirect` is executed.
    ElemUninitialized,

    /// Attempt to divide by zero.
    ///
    /// This trap typically can happen if `div` or `rem` is executed with
    /// zero as divider.
    DivisionByZero,

    /// Attempt to make a conversion to an int failed.
    ///
    /// This can happen when:
    ///
    /// - trying to do signed division (or get the remainder) -2<sup>N-1</sup> over -1. This is
    ///   because the result +2<sup>N-1</sup> isn't representable as a N-bit signed integer.
    /// - trying to truncate NaNs, infinity, or value for which the result is out of range into an integer.
    InvalidConversionToInt,

    /// Stack overflow.
    ///
    /// This is likely caused by some infinite or very deep recursion.
    /// Extensive inlining might also be the cause of stack overflow.
    StackOverflow,

    /// Attempt to invoke a function with mismatching signature.
    ///
    /// This can happen if [`FuncInstance`] was invoked
    /// with mismatching [signature][`Signature`].
    ///
    /// This can always happen with indirect calls. `call_indirect` instruction always
    /// specifies the expected signature of function. If `call_indirect` is executed
    /// with index that points on function with signature different that is
    /// expected by this `call_indirect`, this trap is raised.
    ///
    /// [`Signature`]: struct.Signature.html
    UnexpectedSignature,
    // Error specified by the host.
    //
    // Typically returned from an implementation of [`Externals`].
    //
    // [`Externals`]: trait.Externals.html
    //Host(Box<host::HostError>),
}

/// Internal interpreter error.
#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    /// Module validation error. Might occur only at load time.
    Validation(String),
    /// Error while instantiating a module. Might occur when provided
    /// with incorrect exports (i.e. linkage failure).
    Instantiation(String),
    /// Function-level error.
    Function(String),
    /// Table-level error.
    Table(String),
    /// Memory-level error.
    Memory(String),
    /// Global-level error.
    Global(String),
    /// Value-level error.
    Value(String),
    /// Trap.
    Trap(Trap),
    // Custom embedder error.
    //Host(Box<host::HostError>),
}

impl Into<String> for Error {
    fn into(self) -> String {
        match self {
            Error::Validation(s) => s,
            Error::Instantiation(s) => s,
            Error::Function(s) => s,
            Error::Table(s) => s,
            Error::Memory(s) => s,
            Error::Global(s) => s,
            Error::Value(s) => s,
            Error::Trap(s) => format!("trap: {:?}", s),
            //            Error::Host(e) => format!("user: {}", e),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Validation(ref s) => write!(f, "Validation: {}", s),
            Error::Instantiation(ref s) => write!(f, "Instantiation: {}", s),
            Error::Function(ref s) => write!(f, "Function: {}", s),
            Error::Table(ref s) => write!(f, "Table: {}", s),
            Error::Memory(ref s) => write!(f, "Memory: {}", s),
            Error::Global(ref s) => write!(f, "Global: {}", s),
            Error::Value(ref s) => write!(f, "Value: {}", s),
            Error::Trap(ref s) => write!(f, "Trap: {:?}", s),
            //            Error::Host(ref e) => write!(f, "User: {}", e),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Validation(ref s) => s,
            Error::Instantiation(ref s) => s,
            Error::Function(ref s) => s,
            Error::Table(ref s) => s,
            Error::Memory(ref s) => s,
            Error::Global(ref s) => s,
            Error::Value(ref s) => s,
            Error::Trap(_) => "Trap",
            //            Error::Host(_) => "Host error",
        }
    }
}

//impl<U> From<U> for Error where U: host::HostError + Sized {
//    fn from(e: U) -> Self {
//        Error::Host(Box::new(e))
//    }
//}
//
//impl<U> From<U> for Trap where U: host::HostError + Sized {
//    fn from(e: U) -> Self {
//        Trap::new(TrapKind::Host(Box::new(e)))
//    }
//}

impl From<Trap> for Error {
    fn from(e: Trap) -> Error {
        Error::Trap(e)
    }
}

impl From<TrapKind> for Trap {
    fn from(e: TrapKind) -> Trap {
        Trap::new(e)
    }
}

/// Type of a value.
///
/// See [`RuntimeValue`] for details.
///
/// [`RuntimeValue`]: enum.RuntimeValue.html
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueType {
    /// 32-bit signed or unsigned integer.
    I32,
    /// 64-bit signed or unsigned integer.
    I64,
    /// 32-bit IEEE 754-2008 floating point number.
    F32,
    /// 64-bit IEEE 754-2008 floating point number.
    F64,
}

/// Runtime representation of a value.
///
/// Wasm code manipulate values of the four basic value types:
/// integers and floating-point (IEEE 754-2008) data of 32 or 64 bit width each, respectively.
///
/// There is no distinction between signed and unsigned integer types. Instead, integers are
/// interpreted by respective operations as either unsigned or signed in two’s complement representation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RuntimeValue {
    /// Value of 32-bit signed or unsigned integer.
    I32(i32),
    /// Value of 64-bit signed or unsigned integer.
    I64(i64),
    /// Value of 32-bit IEEE 754-2008 floating point number.
    F32(F32),
    /// Value of 64-bit IEEE 754-2008 floating point number.
    F64(F64),
}

impl RuntimeValue {
    pub fn value_type(&self) -> ValueType {
        match *self {
            RuntimeValue::I32(_) => ValueType::I32,
            RuntimeValue::I64(_) => ValueType::I64,
            RuntimeValue::F32(_) => ValueType::F32,
            RuntimeValue::F64(_) => ValueType::F64,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// Value of 32-bit signed or unsigned integer.
    I32(i32),
    /// Value of 64-bit signed or unsigned integer.
    I64(i64),
    /// Value of 32-bit IEEE 754-2008 floating point number.
    F32(f32),
    /// Value of 64-bit IEEE 754-2008 floating point number.
    F64(f64),
}

pub fn wabt_runtime_value_to_boundary_value(wabt_rv: &wabt::script::Value) -> BoundaryValue {
    match *wabt_rv {
        wabt::script::Value::I32(wabt_rv) => BoundaryValue::I32(wabt_rv),
        wabt::script::Value::I64(wabt_rv) => BoundaryValue::I64(wabt_rv),
        wabt::script::Value::F32(wabt_rv) => BoundaryValue::F32(wabt_rv.to_bits()),
        wabt::script::Value::F64(wabt_rv) => BoundaryValue::F64(wabt_rv.to_bits()),
    }
}

fn boundary_value_to_runtime_value(rv: BoundaryValue) -> RuntimeValue {
    match rv {
        BoundaryValue::I32(bv) => RuntimeValue::I32(bv),
        BoundaryValue::I64(bv) => RuntimeValue::I64(bv),
        BoundaryValue::F32(bv) => RuntimeValue::F32(bv.into()),
        BoundaryValue::F64(bv) => RuntimeValue::F64(bv.into()),
    }
}

pub fn answer_convert(
    res: Result<Option<BoundaryValue>, FaasInterpreterError>,
) -> Result<Option<RuntimeValue>, FaasInterpreterError> {
    match res {
        Ok(None) => Ok(None),
        Ok(Some(rv)) => Ok(Some(boundary_value_to_runtime_value(rv))),
        Err(x) => Err(x),
    }
}

pub fn spec_to_runtime_value(value: wabt::script::Value) -> RuntimeValue {
    match value {
        wabt::script::Value::I32(v) => RuntimeValue::I32(v),
        wabt::script::Value::I64(v) => RuntimeValue::I64(v),
        wabt::script::Value::F32(v) => RuntimeValue::F32(v.into()),
        wabt::script::Value::F64(v) => RuntimeValue::F64(v.into()),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SgxWasmAction {
    Invoke {
        module: Option<String>,
        field: String,
        args: Vec<BoundaryValue>,
    },
    Get {
        module: Option<String>,
        field: String,
    },
    LoadModule {
        name: Option<String>,
        module: Vec<u8>,
    },
    TryLoad {
        module: Vec<u8>,
    },
    Register {
        name: Option<String>,
        as_name: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BoundaryValue {
    I32(i32),
    I64(i64),
    F32(u32),
    F64(u64),
}
