#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
extern crate sgx_tstd as std;

/// Status for Ecall
#[repr(C)]
#[derive(Debug)]
pub struct EnclaveStatus(pub u32);

/// Status for Ocall
pub type UntrustedStatus = EnclaveStatus;

impl EnclaveStatus {
    pub fn default() -> EnclaveStatus {
        EnclaveStatus(0)
    }

    pub fn is_err(&self) -> bool {
        match self.0 {
            0 => false,
            _ => true,
        }
    }

    pub fn is_err_ffi_outbuf(&self) -> bool {
        self.0 == 0x0000_000c
    }
}
