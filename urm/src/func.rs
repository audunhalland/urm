//!
//! Type mapping through function application
//!
//!

use crate::ty::Type;

pub trait Unary {}

pub trait Binary<T, U> {}

#[derive(Debug)]
pub struct Eq<L, R>(pub L, pub R);

impl<L, R> Type for Eq<L, R>
where
    L: Type,
    R: Type<Unit = L::Unit>,
{
    type Unit = bool;
    type Output = bool;
}
