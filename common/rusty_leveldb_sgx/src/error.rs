#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use std::convert::From;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::result;
use std::sync;

use libc::c_int;
use snap;

/// StatusCode describes various failure modes of database operations.
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum StatusCode {
    OK,

    AlreadyExists,
    Corruption,
    CompressionError,
    IOError,
    InvalidArgument,
    InvalidData,
    LockError,
    NotFound,
    NotSupported,
    PermissionDenied,
    Unknown,
    Errno(c_int),
}

/// Status encapsulates a `StatusCode` and an error message. It can be displayed, and also
/// implements `Error`.
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub code: StatusCode,
    pub err: String,
}

impl Default for Status {
    fn default() -> Status {
        Status {
            code: StatusCode::OK,
            err: String::new(),
        }
    }
}

impl Display for Status {
    fn fmt(&self, fmt: &mut Formatter) -> result::Result<(), fmt::Error> {
        fmt.write_str(self.description())
    }
}

impl Error for Status {
    fn description(&self) -> &str {
        &self.err
    }
}

impl Status {
    pub fn new(code: StatusCode, msg: &str) -> Status {
        let err;
        if msg.is_empty() {
            err = format!("{:?}", code)
        } else {
            err = format!("{:?}: {}", code, msg);
        }
        return Status {
            code: code,
            err: err,
        };
    }
}

/// LevelDB's result type
pub type Result<T> = result::Result<T, Status>;

/// err returns a new Status wrapped in a Result.
pub fn err<T>(code: StatusCode, msg: &str) -> Result<T> {
    Err(Status::new(code, msg))
}

impl From<io::Error> for Status {
    fn from(e: io::Error) -> Status {
        let c = match e.kind() {
            io::ErrorKind::NotFound => StatusCode::NotFound,
            io::ErrorKind::InvalidData => StatusCode::Corruption,
            io::ErrorKind::InvalidInput => StatusCode::InvalidArgument,
            io::ErrorKind::PermissionDenied => StatusCode::PermissionDenied,
            _ => StatusCode::IOError,
        };

        Status::new(c, e.description())
    }
}

impl<T> From<sync::PoisonError<T>> for Status {
    fn from(_: sync::PoisonError<T>) -> Status {
        Status::new(StatusCode::LockError, "lock poisoned")
    }
}

impl From<snap::Error> for Status {
    fn from(e: snap::Error) -> Status {
        Status {
            code: StatusCode::CompressionError,
            err: e.description().to_string(),
        }
    }
}
