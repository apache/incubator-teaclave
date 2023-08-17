//! The core library of the policy carrying data implementation. This library contains:
//!
//! * Error types for the PCD
//! * Policy struct definitions

#![cfg_attr(test, allow(unused))]
#![forbid(unsafe_code)]

pub mod ast;
pub mod error;
pub mod expr;
pub mod macros;
pub mod policy;
pub mod types;

#[cfg(test)]
mod test {}
