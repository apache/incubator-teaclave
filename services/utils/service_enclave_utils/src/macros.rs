#[macro_export]
macro_rules! bail {
    ($err:expr $(,)?) => {
        return std::result::Result::Err($err.into());
    };
}

#[macro_export]
macro_rules! ensure {
    ($cond:expr, $err:expr $(,)?) => {
        if !$cond {
            return std::result::Result::Err($err.into());
        }
    };
}
