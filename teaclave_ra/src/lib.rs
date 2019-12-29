#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
#[macro_use]
extern crate sgx_tstd as std;
#[macro_use]
extern crate log;

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

struct SgxQuoteBody;
struct SgxReportBody;

struct SgxQuoteVerifier;

#[macro_use]
mod cert;
#[cfg(feature = "mesalock_sgx")]
pub mod key;

#[cfg(feature = "mesalock_sgx")]
mod ra;
#[cfg(feature = "mesalock_sgx")]
mod ias;
#[cfg(feature = "mesalock_sgx")]
pub use ra::SgxRaReport;

mod quote {
    struct SgxQuote;
}
