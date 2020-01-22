#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
#[macro_use]
extern crate sgx_tstd as std;

use serde::{Deserialize, Serialize};
use teaclave_types::TeaclaveServiceResponseError;

pub trait TeaclaveService<V, U>
where
    U: Serialize + std::fmt::Debug,
    V: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    fn handle_request(&self, request: V) -> std::result::Result<U, TeaclaveServiceResponseError>;
}

pub mod channel;
pub mod config;
pub mod endpoint;
mod protocol;
pub mod server;
mod transport;
mod utils;
