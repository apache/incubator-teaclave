#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
#[macro_use]
extern crate sgx_tstd as std;

use anyhow::{bail, Result};

use log::debug;
use log::error;
use std::backtrace;

#[cfg(feature = "cov")]
use sgx_trts::global_dtors_object;
#[cfg(feature = "cov")]
global_dtors_object! {
    SGX_COV_FINALIZE, sgx_cov_exit = {
        sgx_cov::cov_writeout();
    }
}

pub struct ServiceEnclave;

impl ServiceEnclave {
    pub fn init(name: &str) -> Result<()> {
        env_logger::init();

        debug!("Enclave initializing");

        if backtrace::enable_backtrace(format!("{}.signed.so", name), backtrace::PrintFormat::Full)
            .is_err()
        {
            error!("Cannot enable backtrace");
            bail!("ecall error");
        }

        Ok(())
    }

    pub fn finalize() -> Result<()> {
        debug!("Enclave finalizing");

        Ok(())
    }
}

pub use teaclave_service_enclave_utils_proc_macro::teaclave_service;
