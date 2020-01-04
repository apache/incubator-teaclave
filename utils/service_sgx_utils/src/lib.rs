#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
#[macro_use]
extern crate sgx_tstd as std;

use anyhow::{self, Result};
use teaclave_service_config as config;
pub use teaclave_service_sgx_utils_proc_macro::teaclave_service;

pub fn init_service(name: &str) -> Result<()> {
    use std::backtrace;
    env_logger::init();
    use log::debug;
    use log::error;

    debug!("Enclave [{}]: Initializing...", name);

    if backtrace::enable_backtrace(format!("{}.signed.so", name), backtrace::PrintFormat::Full)
        .is_err()
    {
        error!("Cannot enable backtrace");
        return Err(anyhow::anyhow!("ecall error"));
    }
    if !config::is_runtime_config_initialized() {
        error!("Runtime config is not initialized");
        return Err(anyhow::anyhow!("ecall error"));
    }

    Ok(())
}
