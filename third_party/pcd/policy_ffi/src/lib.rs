#![feature(linkage)]
#![cfg_attr(not(all(feature = "static", not(feature = "modular"))), allow(unused))]

#[cfg(all(feature = "modular", feature = "static"))]
compile_error!("cannot enable both `modular` and `static` features");

#[cfg(all(feature = "modular", not(feature = "static")))]
mod so_impl;
#[cfg(all(feature = "static", not(feature = "modular")))]
mod static_impl;

#[cfg(all(feature = "modular", not(feature = "static")))]
pub use so_impl::*;
#[cfg(all(feature = "static", not(feature = "modular")))]
pub use static_impl::*;

use policy_core::{error::StatusCode, types::OpaquePtr};

// TODO: Unify the signature of the functions.

/// The signature of a generic function symbol.
///
/// # Arguments
///
/// - `args`: The pointer to the argument buffer. A serialized [`FunctionArguments`].
/// - `args_len`: The length of the buffer.
#[cfg_attr(not(feature = "modular"), allow(unused))]
type LibFunction = fn(args: *const u8, args_len: usize) -> StatusCode;

/// The signature of the function symbol that exports the rustc version used to build
/// the library.
///
/// # Arguments
///
/// - `buf`: The caller allocated buffer for holding the string.
/// - `len`: The length of bytes copied to the buffer.
#[cfg_attr(not(feature = "modular"), allow(unused))]
type Version = fn(buf: *const u8, len: *mut usize);

/// The signature of the user defined functions that may apply to the arrays.
///
/// # Arguments
///
/// - `input`: The pointer to the input buffer. A serialized [`FunctionArguments`].
/// - `input_len`: The length of the input buffer.
/// - `output`: The caller-allocated output buffer. A serialized [`FunctionArguments`].
/// - `output_len`: The pointer to the output buffer length. Modified by the callee.
///
/// # Output
/// The output of the user defined function is designed to be as general as possible by using [`FunctionArguments`].
/// It contains the following fields:
///
/// - `output`: The pointer to the [`Box`]-ed trait object.
type UserDefinedFunction =
    fn(input: *const u8, input_len: usize, output: *mut u8, output_len: *mut usize) -> StatusCode;

/// The signature of the function that executes the physical plan via an opaque handle.
///
/// # Arguments
///
/// - `executor`: The opaque pointer to the executor which might be a boxed pointer.
/// - `df`: The output buffer for holding the data frame.
/// - `df_len`: The length of the output buffer len.
#[cfg_attr(not(feature = "modular"), allow(unused))]
type ExecutorFunction = fn(executor: OpaquePtr, df: *mut u8, df_len: *mut usize) -> StatusCode;

/// The signature of the function that creates new executors.
///
/// # Arguments
///
/// - `args`: The pointer to the argument buffer. A serialized [`FunctionArguments`].
/// - `args_len`: The length of the argument buffer.
/// - `p_executor`: The pointer to the created executor. Note that this is a pointer to a smart pointer
///                 [`Box<dyn T>`] where `T` may be [`Sized`], i.e., `p_executor` is `*mut *mut Box<T>`.
///                 Wrapping the executor in a nested pointer is to ensure that fat pointers can be passed
///                 via FFI interfaces. Recall the memory layout of a trait object: it contains a pointer
///                 to the concrete type and a pointer to the vtable for dynamic dispatch.
///                 This pointer is *opaque*.
#[cfg_attr(not(feature = "modular"), allow(unused))]
type ExecutorCreator =
    fn(args: *const u8, args_len: usize, p_executor: *mut OpaquePtr) -> StatusCode;
