use std::fmt::{Debug, Display, Formatter};

use num_enum::{FromPrimitive, IntoPrimitive};

pub type PolicyCarryingResult<T> = std::result::Result<T, PolicyCarryingError>;

/// Enums for the errors that would occur in the implementation of policy carrying data.
#[derive(Clone, Default)]
pub enum PolicyCarryingError {
    /// Already loaded.
    AlreadyLoaded,
    /// Invalid input.
    InvalidInput(String),
    /// Duplicate column names.
    DuplicateColumn(String),
    /// Cannot ser / deserialize.
    SerializeError(String),
    /// An operation is impossible or the operands are in-compatible.
    ImpossibleOperation(String),
    /// Inconsistent policies.
    InconsistentPolicy(String),
    /// Schema mismatch.
    SchemaMismatch(String),
    /// Type error.
    TypeMismatch(String),
    /// Unsupported operation.
    OperationNotSupported(String),
    /// Index out of bound.
    OutOfBound(String),
    /// Privacy error.
    PrivacyError(String),
    /// Not found.
    ColumnNotFound(String),
    /// Filesystem error.
    FsError(String),
    /// Symbol not found.
    SymbolNotFound(String),
    /// Operation not allowed: policy forbids this.
    OperationNotAllowed(String),
    /// Parse failed.
    ParseError(String, String),
    /// Version error.
    VersionMismatch(String),
    /// Unknown error.
    #[default]
    Unknown,
}

/// Status code returned from the external functions.
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, FromPrimitive)]
#[repr(i64)]
pub enum StatusCode {
    Ok = 0,
    Unsupported = 1,
    SerializeError = 2,
    NotLoaded = 3,
    Already = 4,
    ColumnNotFound = 5,
    DuplicateColumn = 6,
    FsError = 7,
    ImpossibleOperation = 8,
    InvalidInput = 9,
    InconsistentPolicy = 10,
    OperationNotAllowed = 11,
    OperationNotSupported = 12,
    OutOfBound = 13,
    ParseError = 14,
    PrivacyError = 15,
    SchemaMismatch = 16,
    SymbolNotFound = 17,
    TypeMismatch = 18,
    VersionMismatch = 19,
    #[default]
    Unknown = 0x100,
}

impl From<i32> for StatusCode {
    fn from(value: i32) -> Self {
        (value as i64).into()
    }
}

impl From<PolicyCarryingError> for StatusCode {
    fn from(value: PolicyCarryingError) -> Self {
        match value {
            PolicyCarryingError::AlreadyLoaded => StatusCode::Already,
            PolicyCarryingError::ColumnNotFound(_) => StatusCode::ColumnNotFound,
            PolicyCarryingError::DuplicateColumn(_) => StatusCode::DuplicateColumn,
            PolicyCarryingError::FsError(_) => StatusCode::FsError,
            PolicyCarryingError::ImpossibleOperation(_) => StatusCode::ImpossibleOperation,
            PolicyCarryingError::InvalidInput(_) => StatusCode::InvalidInput,
            PolicyCarryingError::InconsistentPolicy(_) => StatusCode::InconsistentPolicy,
            PolicyCarryingError::OperationNotAllowed(_) => StatusCode::OperationNotAllowed,
            PolicyCarryingError::OperationNotSupported(_) => StatusCode::OperationNotSupported,
            PolicyCarryingError::OutOfBound(_) => StatusCode::OutOfBound,
            PolicyCarryingError::ParseError(_, _) => StatusCode::ParseError,
            PolicyCarryingError::PrivacyError(_) => StatusCode::PrivacyError,
            PolicyCarryingError::SchemaMismatch(_) => StatusCode::SchemaMismatch,
            PolicyCarryingError::SerializeError(_) => StatusCode::SerializeError,
            PolicyCarryingError::SymbolNotFound(_) => StatusCode::SymbolNotFound,
            PolicyCarryingError::TypeMismatch(_) => StatusCode::TypeMismatch,
            PolicyCarryingError::VersionMismatch(_) => StatusCode::VersionMismatch,
            _ => StatusCode::Unknown,
        }
    }
}

impl From<StatusCode> for PolicyCarryingError {
    fn from(value: StatusCode) -> Self {
        match value {
            StatusCode::Already => PolicyCarryingError::AlreadyLoaded,
            StatusCode::ColumnNotFound => PolicyCarryingError::ColumnNotFound("".into()),
            StatusCode::DuplicateColumn => PolicyCarryingError::DuplicateColumn("".into()),
            StatusCode::FsError => PolicyCarryingError::FsError("".into()),
            StatusCode::ImpossibleOperation => PolicyCarryingError::ImpossibleOperation("".into()),
            StatusCode::OperationNotAllowed => PolicyCarryingError::OperationNotAllowed("".into()),
            StatusCode::OperationNotSupported => {
                PolicyCarryingError::OperationNotSupported("".into())
            }
            StatusCode::OutOfBound => PolicyCarryingError::OutOfBound("".into()),
            StatusCode::ParseError => PolicyCarryingError::ParseError("".into(), "".into()),
            StatusCode::PrivacyError => PolicyCarryingError::PrivacyError("".into()),
            StatusCode::SchemaMismatch => PolicyCarryingError::SchemaMismatch("".into()),
            StatusCode::SerializeError => PolicyCarryingError::SerializeError("".into()),
            StatusCode::TypeMismatch => PolicyCarryingError::TypeMismatch("".into()),
            StatusCode::VersionMismatch => PolicyCarryingError::VersionMismatch("".into()),
            _ => PolicyCarryingError::Unknown,
        }
    }
}

impl Debug for PolicyCarryingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for PolicyCarryingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyLoaded => write!(f, "already loaded"),
            Self::ImpossibleOperation(info) => write!(f, "This operation is impossible: {}", info),
            Self::DuplicateColumn(name) => write!(f, "Duplicate column name found: {}", name),
            Self::SchemaMismatch(info) => write!(f, "Schema mismatch: {}", info),
            Self::InconsistentPolicy(info) => write!(f, "Inconsistent policies: {}", info),
            Self::InvalidInput(info) => write!(f, "invalid input: {}", info),
            Self::VersionMismatch(ver) => write!(f, "This version {} is not supported", ver),
            Self::TypeMismatch(info) => write!(f, "Type mismatch: {}", info),
            Self::ColumnNotFound(info) => write!(f, "Missing column {}", info),
            Self::SerializeError(info) => write!(f, "Ser- / deserialization error: {}", info),
            Self::OutOfBound(info) => write!(f, "Index out of bound: {}", info),
            Self::OperationNotSupported(info) => write!(f, "Operation not supported: {}", info),
            Self::FsError(info) => write!(f, "IO error: {}", info),
            Self::OperationNotAllowed(info) => write!(f, "Operation not allowed: {}", info),
            Self::SymbolNotFound(info) => write!(f, "Symbol not found for {}", info),
            Self::PrivacyError(info) => {
                write!(f, "Privacy scheme encountered a fatal error: {}", info)
            }
            Self::ParseError(file, info) => write!(f, "Cannot parse {}, {}", file, info),
            Self::Unknown => write!(
                f,
                "Unknown error. This may be due to some implementation bugs"
            ),
        }
    }
}

impl std::error::Error for PolicyCarryingError {}
