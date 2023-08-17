#[macro_export]
macro_rules! get_lock {
    ($lock:expr, $op:ident) => {
        match $lock.$op() {
            Ok(lock) => lock,
            Err(_) => return Err($crate::error::PolicyCarryingError::Unknown),
        }
    };
}

#[macro_export]
macro_rules! args {
    ($($key:tt : $value:expr),* $(,)?) => {{
        let mut arg = $crate::types::FunctionArguments {
            inner: Default::default(),
        };

        $(
            arg.inner.insert($key.into(), $value.into());
        )*

        arg
    }};
}

/// Ensures a condition must hold or returns a failure indicating the reason.
#[macro_export]
macro_rules! pcd_ensures {
    ($cond:expr, $variant:ident: $($tt:tt)+) => {
        if !$cond {
            return Err($crate::error::PolicyCarryingError::$variant(format!(
                $($tt)+
            )));
        }
    };

    ($cond:expr, $variant:ident) => {
        if !$cond {
            return Err($crate::error::PolicyCarryingError::$variant);
        }
    };
}

/// Helper for converting a [`PolicyCarryingResult`](crate::error::PolicyCarryingResult)
/// into a [`StatusCode`](crate::error::StatusCode).
#[macro_export]
macro_rules! pcd_ffi_try {
    ($op:expr) => {
        match $op {
            Ok(val) => val,
            Err(err) => return $crate::error::StatusCode::from(err),
        }
    };
}
