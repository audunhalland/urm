//!
//! Quantification of types.
//!
//! In this context it means a type-mapping of the Self type
//! into either Self or some collection of Self.
//!

/// Quantify some type.
pub trait Quantify<U> {
    type Output;
}

/// Quantify a type as itself, i.e. no quantification.
pub struct AsSelf;

impl<U> Quantify<U> for AsSelf {
    type Output = U;
}

/// Quantify a type using `Option<_>`.
pub struct AsOption;

impl<U> Quantify<U> for AsOption {
    type Output = Option<U>;
}

/// Quantify a type using a `Vec<_>`.
pub struct AsVec;

impl<U> Quantify<U> for AsVec {
    type Output = Vec<U>;
}
