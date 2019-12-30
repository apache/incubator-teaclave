#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
#[macro_use]
extern crate sgx_tstd as std;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RaError {
    #[error("OCall failed")]
    OCallError,
    #[error("Ias error")]
    IasError,
    #[error("Get quote error")]
    QuoteError,
}

// struct SgxQuoteVerifier;

#[macro_use]
mod cert;
pub mod quote;

use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "mesalock_sgx")]  {
        pub mod key;
        mod ra;
        mod ias;
        pub use ra::SgxRaReport;
    } else {
    }
}
