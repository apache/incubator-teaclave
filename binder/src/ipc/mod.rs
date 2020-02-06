use serde::{Deserialize, Serialize};
use std::prelude::v1::*;

// Intra-Process-Communication
// Developer should split a process into two partitions, App and TEE.

// Caller of an IPC function
// Generic U: ArgmentsInfo type
// Generic V: ReturnInfo type
pub trait IpcSender {
    fn invoke<U, V>(&mut self, cmd: u32, input: U) -> std::result::Result<V, IpcError>
    where
        U: Serialize,
        V: for<'de> Deserialize<'de>;
}

// Service Instance of an IPC function
// Generic U: ArgmentsInfo type
// Generic V: ReturnInfo type
pub trait IpcService<U, V>
where
    U: for<'de> Deserialize<'de>,
    V: Serialize,
{
    fn handle_invoke(&self, input: U) -> teaclave_types::TeeServiceResult<V>;
}

// Callee of an IPC function
// Generic U: ArgmentsInfo type
// Generic V: ReturnInfo type
// Generic X: Service Instance that serves of <U, V>
// Dispatch a received input to a specific service instance.
pub trait IpcReceiver {
    fn dispatch<U, V, X>(input: &[u8], x: X) -> anyhow::Result<Vec<u8>>
    where
        U: for<'de> Deserialize<'de>,
        V: Serialize,
        X: IpcService<U, V>;
}

#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
pub enum IpcError {
    #[error("SgxError")]
    SgxError(i32),
    #[error("EnclaveError")]
    EnclaveError(teaclave_types::EnclaveStatus),
    #[error("SerdeError")]
    SerdeError,
    #[error("TeeServiceError")]
    TeeServiceError,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "app")]  {
        mod app;
        pub use app::ECallChannel;
    } else if #[cfg(feature = "mesalock_sgx")] {
        mod enclave;
        pub use enclave::ECallReceiver;
    }
}
