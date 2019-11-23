// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use std::fmt;
use std::{io, net};

pub type Result<T> = std::result::Result<T, Error>;

/// Status for Ecall
#[repr(C)]
pub struct EnclaveStatus(u32);

/// Status for Ocall
pub type UntrustedStatus = EnclaveStatus;

pub struct Error {
    repr: Repr,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.repr, f)
    }
}

enum Repr {
    Simple(ErrorKind),
    Custom(Box<Custom>),
}

#[derive(Debug)]
struct Custom {
    kind: ErrorKind,
    error: Box<dyn std::error::Error + Send + Sync>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    /// The operation lacked the necessary privileges to complete.
    PermissionDenied,
    /// Expecting a value from option but get None.
    MissingValue,
    /// The connection was refused by the remote.
    ConnectionRefused,
    /// Internal error during Remote Attestation.
    RAInternalError,
    /// Invalid connection refused during Remote Attestion .
    InvalidRA,
    /// Error occurred during Internal RPC, happens inside SDK.
    InternalRPCError,
    /// Error when sending a request, happens outside SDK.
    RPCRequestError,
    /// Error when receiving a response, happens outside SDK.
    RPCResponseError,
    /// Implementation errors.
    BadImplementation,
    /// Error parsing HTTP.
    InvalidHTTPRequest,
    /// Failed to parse
    ParseError,
    /// FFI caller provided outbuf too small
    FFICallerOutBufferTooSmall,
    /// TEE doesn't support this command
    ECallCommandNotRegistered,
    /// IO Error,
    IoError,
    /// ECall Error.
    ECallError,
    /// OCall Error.
    OCallError,
    /// TCP Error.
    TCPError,
    /// TLS Error.
    TLSError,
    /// Error occurred when retrieving system time.
    SystemTimeError,
    /// Error occurred when doing UUID operations.
    UUIDError,
    /// Cyprto related errors.
    CryptoError,
    /// Error when doing sync primitves operations, e.g. RWLock.
    SyncPrimitiveError,
    /// Error when doing ecall/ocall.
    SgxError,
    /// Error in untrusted app.
    UntrustedAppError,
    /// Error during ffi data converting.
    FFIError,
    /// Expect to get key from KMS but failed
    KeyNotFoundError,
    /// Function is not supported
    FunctionNotSupportedError,
    /// Input provided by users is invalid
    InvalidInputError,
    /// Worker failed to generate output
    OutputGenerationError,
    /// IPC error
    IPCError,
    /// IAS client key or cert not available
    IASClientKeyCertError,
    /// No valid worker for the task
    NoValidWorkerError,
    /// RPC Message size excceds the limit
    MsgSizeLimitExceedError,
    /// Unhandled MesaPy exception encountered
    MesaPyError,
    /// Output from server is invalid
    InvalidOutputError,
    /// Others.
    Unknown,
}

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
        match Error::from(self.0).kind() {
            ErrorKind::FFICallerOutBufferTooSmall => true,
            _ => false,
        }
    }
}

impl From<Result<()>> for EnclaveStatus {
    #[inline]
    fn from(r: Result<()>) -> EnclaveStatus {
        match r {
            Ok(_) => EnclaveStatus(0),
            Err(e) => EnclaveStatus(e.into()),
        }
    }
}

impl ErrorKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ErrorKind::PermissionDenied => "permission denied",
            ErrorKind::MissingValue => "missing value",
            ErrorKind::ConnectionRefused => "connection refused",
            ErrorKind::RAInternalError => "ra internal error",
            ErrorKind::RPCRequestError => "rpc request error",
            ErrorKind::RPCResponseError => "rpc response error",
            ErrorKind::InvalidRA => "invalid ra",
            ErrorKind::InternalRPCError => "internal rpc error",
            ErrorKind::BadImplementation => "invalid implementation",
            ErrorKind::InvalidHTTPRequest => "invalid http request",
            ErrorKind::ParseError => "failed to parse",
            ErrorKind::FFICallerOutBufferTooSmall => "ffi caller out buffer too small",
            ErrorKind::ECallCommandNotRegistered => "ecall command not registered",
            ErrorKind::IoError => "io error",
            ErrorKind::ECallError => "ecall error",
            ErrorKind::OCallError => "ocall error",
            ErrorKind::TCPError => "tcp error",
            ErrorKind::TLSError => "tls error",
            ErrorKind::SystemTimeError => "system time error",
            ErrorKind::UUIDError => "uuid error",
            ErrorKind::CryptoError => "crypto error",
            ErrorKind::SyncPrimitiveError => "sync primitive error",
            ErrorKind::SgxError => "sgx error",
            ErrorKind::UntrustedAppError => "untrusted app error",
            ErrorKind::FFIError => "ffi error",
            ErrorKind::KeyNotFoundError => "key not found error",
            ErrorKind::FunctionNotSupportedError => "function not supported error",
            ErrorKind::InvalidInputError => "invalid user input error",
            ErrorKind::OutputGenerationError => "generate output error",
            ErrorKind::IPCError => "ipc error",
            ErrorKind::IASClientKeyCertError => {
                "intel attestation service client key/certificate unavailable error"
            }
            ErrorKind::NoValidWorkerError => "no valid worker error",
            ErrorKind::MsgSizeLimitExceedError => "message size exceeds limit",
            ErrorKind::MesaPyError => "unhandled mesapy exception",
            ErrorKind::InvalidOutputError => "invalid rpc output",
            ErrorKind::Unknown => "unknown error",
        }
    }
}

impl From<ErrorKind> for Error {
    #[inline]
    fn from(kind: ErrorKind) -> Error {
        Error {
            repr: Repr::Simple(kind),
        }
    }
}

impl From<u32> for Error {
    #[inline]
    fn from(kind: u32) -> Error {
        let err_kind = match kind {
            0x0000_0001 => ErrorKind::PermissionDenied,
            0x0000_0002 => ErrorKind::MissingValue,
            0x0000_0003 => ErrorKind::ConnectionRefused,
            0x0000_0004 => ErrorKind::RAInternalError,
            0x0000_0005 => ErrorKind::InvalidRA,
            0x0000_0006 => ErrorKind::InternalRPCError,
            0x0000_0007 => ErrorKind::RPCRequestError,
            0x0000_0008 => ErrorKind::RPCResponseError,
            0x0000_0009 => ErrorKind::BadImplementation,
            0x0000_000a => ErrorKind::InvalidHTTPRequest,
            0x0000_000b => ErrorKind::ParseError,
            0x0000_000c => ErrorKind::FFICallerOutBufferTooSmall,
            0x0000_000d => ErrorKind::ECallCommandNotRegistered,
            0x0000_1000 => ErrorKind::IoError,
            0x0000_1001 => ErrorKind::ECallError,
            0x0000_1002 => ErrorKind::OCallError,
            0x0000_1003 => ErrorKind::TCPError,
            0x0000_1004 => ErrorKind::TLSError,
            0x0000_1005 => ErrorKind::SystemTimeError,
            0x0000_1006 => ErrorKind::UUIDError,
            0x0000_1007 => ErrorKind::CryptoError,
            0x0000_1008 => ErrorKind::SyncPrimitiveError,
            0x0000_1009 => ErrorKind::SgxError,
            0x0000_100a => ErrorKind::UntrustedAppError,
            0x0000_100b => ErrorKind::FFIError,
            0x0000_100c => ErrorKind::KeyNotFoundError,
            0x0000_100d => ErrorKind::FunctionNotSupportedError,
            0x0000_100e => ErrorKind::InvalidInputError,
            0x0000_100f => ErrorKind::OutputGenerationError,
            0x0000_1010 => ErrorKind::IPCError,
            0x0000_1011 => ErrorKind::IASClientKeyCertError,
            0x0000_1012 => ErrorKind::NoValidWorkerError,
            0x0000_1013 => ErrorKind::MsgSizeLimitExceedError,
            0x0000_1014 => ErrorKind::MesaPyError,
            0x0000_1015 => ErrorKind::InvalidOutputError,
            _ => ErrorKind::Unknown,
        };

        Error {
            repr: Repr::Simple(err_kind),
        }
    }
}

impl Into<u32> for Error {
    #[inline]
    fn into(self) -> u32 {
        match self.kind() {
            ErrorKind::PermissionDenied => 0x0000_0001,
            ErrorKind::MissingValue => 0x0000_0002,
            ErrorKind::ConnectionRefused => 0x0000_0003,
            ErrorKind::RAInternalError => 0x0000_0004,
            ErrorKind::InvalidRA => 0x0000_0005,
            ErrorKind::InternalRPCError => 0x0000_0006,
            ErrorKind::RPCRequestError => 0x0000_0007,
            ErrorKind::RPCResponseError => 0x0000_0008,
            ErrorKind::BadImplementation => 0x0000_0009,
            ErrorKind::InvalidHTTPRequest => 0x0000_000a,
            ErrorKind::ParseError => 0x0000_000b,
            ErrorKind::FFICallerOutBufferTooSmall => 0x0000_000c,
            ErrorKind::ECallCommandNotRegistered => 0x0000_000d,

            ErrorKind::IoError => 0x0000_1000,
            ErrorKind::ECallError => 0x0000_1001,
            ErrorKind::OCallError => 0x0000_1002,
            ErrorKind::TCPError => 0x0000_1003,
            ErrorKind::TLSError => 0x0000_1004,
            ErrorKind::SystemTimeError => 0x0000_1005,
            ErrorKind::UUIDError => 0x0000_1006,
            ErrorKind::CryptoError => 0x0000_1007,
            ErrorKind::SyncPrimitiveError => 0x0000_1008,
            ErrorKind::SgxError => 0x0000_1009,
            ErrorKind::UntrustedAppError => 0x0000_100a,
            ErrorKind::FFIError => 0x0000_100b,
            ErrorKind::KeyNotFoundError => 0x0000_100c,
            ErrorKind::FunctionNotSupportedError => 0x0000_100d,
            ErrorKind::InvalidInputError => 0x0000_100e,
            ErrorKind::OutputGenerationError => 0x0000_100f,
            ErrorKind::IPCError => 0x0000_1010,
            ErrorKind::IASClientKeyCertError => 0x0000_1011,
            ErrorKind::NoValidWorkerError => 0x0000_1012,
            ErrorKind::MsgSizeLimitExceedError => 0x0000_1013,
            ErrorKind::MesaPyError => 0x0000_1014,
            ErrorKind::InvalidOutputError => 0x0000_1015,
            ErrorKind::Unknown => 0xffff_ffff,
        }
    }
}

impl Error {
    pub fn new<E>(kind: ErrorKind, error: E) -> Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::_new(kind, error.into())
    }

    fn _new(kind: ErrorKind, error: Box<dyn std::error::Error + Send + Sync>) -> Error {
        Error {
            repr: Repr::Custom(Box::new(Custom { kind, error })),
        }
    }

    pub fn get_ref(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::Custom(ref c) => Some(&*c.error),
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut (dyn std::error::Error + Send + Sync + 'static)> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::Custom(ref mut c) => Some(&mut *c.error),
        }
    }

    pub fn into_inner(self) -> Option<Box<dyn std::error::Error + Send + Sync>> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::Custom(c) => Some(c.error),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            Repr::Custom(ref c) => c.kind,
            Repr::Simple(kind) => kind,
        }
    }

    pub fn unknown() -> Error {
        Error::from(ErrorKind::Unknown)
    }
}

impl fmt::Debug for Repr {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Repr::Custom(ref c) => fmt::Debug::fmt(&c, fmt),
            Repr::Simple(kind) => fmt.debug_tuple("Kind").field(&kind).finish(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.repr {
            Repr::Custom(ref c) => c.error.fmt(fmt),
            Repr::Simple(kind) => write!(fmt, "{}", kind.as_str()),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::Custom(ref c) => c.error.source(),
        }
    }
}

impl From<io::Error> for Error {
    #[inline]
    fn from(err: io::Error) -> Error {
        Error::new(ErrorKind::IoError, err)
    }
}

impl From<serde_json::Error> for Error {
    #[inline]
    fn from(err: serde_json::Error) -> Error {
        Error::new(ErrorKind::ParseError, err)
    }
}

impl From<net::AddrParseError> for Error {
    #[inline]
    fn from(err: net::AddrParseError) -> Error {
        Error::new(ErrorKind::ParseError, err)
    }
}

impl From<EnclaveStatus> for Error {
    #[inline]
    fn from(status: EnclaveStatus) -> Error {
        Error::from(status.0)
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    #[inline]
    fn from(_err: std::sync::PoisonError<T>) -> Error {
        Error::from(ErrorKind::SyncPrimitiveError)
    }
}

impl From<Error> for EnclaveStatus {
    #[inline]
    fn from(err: Error) -> EnclaveStatus {
        EnclaveStatus(err.into())
    }
}

impl From<std::str::Utf8Error> for Error {
    #[inline]
    fn from(_err: std::str::Utf8Error) -> Error {
        Error::from(ErrorKind::ParseError)
    }
}

use sgx_types;
impl From<sgx_types::sgx_status_t> for Error {
    #[inline]
    fn from(status: sgx_types::sgx_status_t) -> Error {
        Error::new(ErrorKind::SgxError, SgxStatus::from(status))
    }
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Debug)]
pub enum SgxStatus {
    Inner(sgx_types::sgx_status_t),
}

impl fmt::Display for SgxStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SgxStatus::Inner(x) => write!(f, "{}", x.as_str()),
        }
    }
}

impl From<sgx_types::sgx_status_t> for SgxStatus {
    #[inline]
    fn from(status: sgx_types::sgx_status_t) -> SgxStatus {
        SgxStatus::Inner(status)
    }
}

impl std::error::Error for SgxStatus {
    fn description(&self) -> &str {
        match self {
            SgxStatus::Inner(x) => x.as_str(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Custom, Error, ErrorKind, Repr};
    use std::boxed::Box;
    use std::fmt;

    #[test]
    fn test_debug_error() {
        let err = Error {
            repr: Repr::Custom(Box::new(Custom {
                kind: ErrorKind::Unknown,
                error: Box::new(Error {
                    repr: super::Repr::Simple(ErrorKind::Unknown),
                }),
            })),
        };
        let expected = "Custom { \
                        kind: Unknown, \
                        error: Kind(Unknown) \
                        }"
        .to_string();
        assert_eq!(format!("{:?}", err), expected);
    }

    #[test]
    fn test_downcasting() {
        #[derive(Debug)]
        struct TestError;

        impl fmt::Display for TestError {
            fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
                Ok(())
            }
        }

        impl std::error::Error for TestError {
            fn description(&self) -> &str {
                "asdf"
            }
        }

        // we have to call all of these UFCS style right now since method
        // resolution won't implicitly drop the Send+Sync bounds
        let mut err = Error::new(ErrorKind::Unknown, TestError);
        assert!(err.get_ref().unwrap().is::<TestError>());
        assert_eq!("asdf", err.get_ref().unwrap().description());
        assert!(err.get_mut().unwrap().is::<TestError>());
        let extracted = err.into_inner().unwrap();
        extracted.downcast::<TestError>().unwrap();
    }
}
