//! A crate for working around the algebraic structures and is tailored to the need to policy encoding and
//! it enforcement. Specifically, we aim to implement the following structures:
//!
//! * Semiring
//! * Module on a semiring
//! * Group / communitative monoid
//!
//! Unfortunately, other implementations are simply out of scope.

#![forbid(unsafe_code)]

use std::{
    fmt::{Debug, Display},
    ops::{Add, Mul},
};

#[cfg(test)]
mod test {
    #[test]
    fn play() {
        
    }
}

/// Defines an additive identity element for `Self`.
///
/// # Laws
///
/// ```{.text}
/// a + 0 = a       ∀ a ∈ Self
/// 0 + a = a       ∀ a ∈ Self
/// ```
pub trait Zero: Sized + Add<Self, Output = Self> {
    /// Returns the additive identity element of `Self`, `0`.
    /// # Purity
    ///
    /// This function should return the same result at all times regardless of
    /// external mutable state, for example values stored in TLS or in
    /// `static mut`s.
    // This cannot be an associated constant, because of bignums.
    fn zero() -> Self;

    /// Sets `self` to the additive identity element of `Self`, `0`.
    fn set_zero(&mut self) {
        *self = Zero::zero();
    }

    /// Returns `true` if `self` is equal to the additive identity.
    fn is_zero(&self) -> bool;
}

/// Defines a multiplicative identity element for `Self`.
///
/// # Laws
///
/// ```{.text}
/// a * 1 = a       ∀ a ∈ Self
/// 1 * a = a       ∀ a ∈ Self
/// ```
pub trait One: Sized + Mul<Self, Output = Self> {
    /// Returns the multiplicative identity element of `Self`, `1`.
    ///
    /// # Purity
    ///
    /// This function should return the same result at all times regardless of
    /// external mutable state, for example values stored in TLS or in
    /// `static mut`s.
    // This cannot be an associated constant, because of bignums.
    fn one() -> Self;

    /// Sets `self` to the multiplicative identity element of `Self`, `1`.
    fn set_one(&mut self) {
        *self = One::one();
    }

    /// Returns `true` if `self` is equal to the multiplicative identity.
    ///
    /// For performance reasons, it's best to implement this manually.
    /// After a semver bump, this method will be required, and the
    /// `where Self: PartialEq` bound will be removed.
    #[inline]
    fn is_one(&self) -> bool
    where
        Self: PartialEq,
    {
        *self == Self::one()
    }
}

/// A monoid. In abstract algebra, a branch of mathematics, a monoid is a set equipped with an associative binary operation and an identity element.
pub trait Monoid:
    'static + Copy + Clone + Debug + Display + Default + Send + Sync + Add<Self, Output = Self> + Zero
{
    type Element: Sized + Copy + Clone + Debug + Display + Default + Send + Sync;

    /// The identity element.
    const ZERO: Self;
}

/// A semigroup is a quasigroup that is **associative**.
///
/// A semigroup is a set equipped with a closed associative binary operation and that has the divisibility property.
pub trait SemiGroup {}

macro_rules! impl_zero {
    ($ty:ident, $zero:tt) => {
        impl Zero for $ty {
            fn zero() -> Self {
                $zero
            }

            fn is_zero(&self) -> bool {
                self == &$zero
            }
        }
    };
}

macro_rules! impl_one {
    ($ty:ident, $one:tt) => {
        impl One for $ty {
            fn one() -> Self {
                $one
            }

            fn is_one(&self) -> bool {
                self == &$one
            }
        }
    };
}

impl_zero!(i8, 0);
impl_zero!(i16, 0);
impl_zero!(i32, 0);
impl_zero!(i64, 0);
impl_zero!(u8, 0);
impl_zero!(u16, 0);
impl_zero!(u32, 0);
impl_zero!(u64, 0);
impl_zero!(f32, 0.0);
impl_zero!(f64, 0.0);

impl_one!(i8, 1);
impl_one!(i16, 1);
impl_one!(i32, 1);
impl_one!(i64, 1);
impl_one!(u8, 1);
impl_one!(u16, 1);
impl_one!(u32, 1);
impl_one!(u64, 1);
impl_one!(f32, 1.0);
impl_one!(f64, 1.0);

pub trait SemiRing:
    'static
    + Copy
    + Clone
    + Debug
    + Display
    + Default
    + Send
    + Sync
    + Add<Self, Output = Self>
    + Zero
    + One
{
    type Element: Sized + Copy + Clone + Debug + Display + Default + Send + Sync;

    /// The identity element.
    const ZERO: Self;
    /// The identity element for multiplication.
    const ONE: Self;
}
