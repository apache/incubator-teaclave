// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Monadic mayfail notation for chained error handling

/// maiyfail use duck typing.
///
/// Syntax:
/// `(instr)*; ret expr`
///
/// instr can be:
///
/// * `pattern =<< expression`: unbox the value as `pattern` from `expression`.
/// `expression` can be converted to a monadic value of type `M<T> through a
/// `to_mt_result` method. `M<T>` should support a monadic bind operation
/// `and_then<U, F>(self, f: F)-> M<U> where F: FnOnce(T)->M<U>`. `expression`s
/// in the same mayfail! block should be convertable to the same `M<T>`.
///
/// * `let pattern = expression`: assign expression to pattern, as
/// normal rust let.
///
/// The mayfial module provides the `MayfailNop` trait to trivially convert
/// `std::option::Option<T>` and `std::result::Result<T, E>` to `mesatee_core::Result<T>`.
/// Users can define their own traits to implement `to_mt_result` for types
/// appearing in `mayfail!` blocks.
///
/// Example:
///
/// ```
/// fn main() {
///     use mesatee_core::mayfail::MayfailNop;
///
///     let result_x: Option<i32> = Some(1);
///     let ret = mayfail! {
///     let n = 3;
///         x =<< result_x;
///         y =<< Some(n);
///         ret x + y
///     };
///     assert!(ret.is_ok() && ret.unwrap() == 4);
/// }
/// ```
macro_rules! mayfail {
    (let $p: pat = $e: expr ; $($t: tt)*) => (
        { let $p = $e; mayfail! { $($t)* } }
    );
    (let $p: ident : $ty: ty = $e: expr; $($t: tt)*) => (
        { let $p: $ty = $e; mayfail! { $($t)* } }
    );
    ($p: pat =<< $e: expr; $($t: tt)*) => (
        ($e).to_mt_result(file!(), line!()).and_then(move |$p| mayfail! { $($t)* })
    );
    ($p: ident : $ty: ty =<< $e: expr ; $($t: tt)*) => (
        ($e).to_mt_result(file!(), line!()).and_then(move |$p : $ty| mayfail! { $($t)* })
    );
    (ret $f: expr) => (Ok($f))
}

use crate::Error;
use crate::ErrorKind;
use crate::Result;

pub trait MayfailNop<T> {
    fn to_mt_result(self: Self, file: &'static str, line: u32) -> Result<T>;
}

impl<T> MayfailNop<T> for Option<T> {
    #[inline]
    fn to_mt_result(self: Self, _file: &'static str, _line: u32) -> Result<T> {
        self.ok_or_else(|| Error::from(ErrorKind::MissingValue))
    }
}

impl<T, E> MayfailNop<T> for std::result::Result<T, E> {
    #[inline]
    fn to_mt_result(self: Self, _file: &'static str, _line: u32) -> Result<T> {
        self.map_err(|_| Error::unknown())
    }
}

pub trait MayfailTrace<T> {
    fn to_mt_result(self: Self, file: &'static str, line: u32) -> Result<T>;
}

impl<T> MayfailTrace<T> for Option<T> {
    #[inline]
    fn to_mt_result(self: Self, file: &'static str, line: u32) -> Result<T> {
        self.ok_or_else(|| {
            trace!("error at {}:{}", file, line);
            Error::from(ErrorKind::MissingValue)
        })
    }
}

impl<T, E> MayfailTrace<T> for std::result::Result<T, E> {
    #[inline]
    default fn to_mt_result(self: Self, file: &'static str, line: u32) -> Result<T> {
        self.map_err(|_| {
            trace!("error at {}:{}", file, line);
            Error::unknown()
        })
    }
}

impl<T> MayfailTrace<T> for Result<T> {
    #[inline]
    fn to_mt_result(self: Self, _file: &'static str, _line: u32) -> Result<T> {
        self
    }
}

#[cfg(test)]
mod test {
    use crate::ErrorKind;

    #[test]
    fn test_mayfail_option() {
        let ret = mayfail! {
            let n = 3;
            x =<< Some(1);
            y =<< Some(n);
            ret x + y
        };
        assert!(ret.is_ok() && ret.unwrap() == 4);

        let ret = mayfail! {
            let n: Option<i32> = None;
            x =<< Some(1i32);
            y =<< n;
            ret x + y
        };
        assert!(ret.is_err());
        assert_eq!(ret.unwrap_err().kind(), ErrorKind::MissingValue);
    }

    #[derive(Debug)]
    struct SomeThirdPartyError;

    impl std::fmt::Display for SomeThirdPartyError {
        fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Ok(())
        }
    }

    impl std::error::Error for SomeThirdPartyError {
        fn description(&self) -> &str {
            "test"
        }
    }

    type TestResult<T> = std::result::Result<T, SomeThirdPartyError>;

    use super::MayfailNop;

    #[test]
    fn test_mayfail_result() {
        let result_x: TestResult<i32> = Ok(1);
        let ret = mayfail! {
            let n = 3;
            x =<< result_x;
            y =<< TestResult::<i32>::Ok(n);
            ret x + y
        };
        assert!(ret.is_ok() && ret.unwrap() == 4);

        let result_x: TestResult<i32> = Ok(1);
        let ret = mayfail! {
            let err = SomeThirdPartyError;
            x =<< result_x;
            y =<< TestResult::<i32>::Err(err);
            ret x + y
        };
        assert!(ret.is_err());
        assert_eq!(ret.unwrap_err().kind(), ErrorKind::Unknown);
    }

    #[test]
    fn test_mayfail_mix() {
        let result_x: TestResult<i32> = Ok(1);
        let ret = mayfail! {
            x =<< result_x;
            y =<< Option::<i32>::None;
            ret x + y
        };
        assert!(ret.is_err());
        assert_eq!(ret.unwrap_err().kind(), ErrorKind::MissingValue);
    }
}
