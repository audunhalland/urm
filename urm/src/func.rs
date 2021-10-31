//!
//! Type mapping through function application
//!
//!

use crate::ty::{Bool, Type, Typed};

pub trait Unary {}

pub trait Binary<T, U> {}

#[derive(Debug)]
pub struct Eq<L, R>(pub L, pub R);

impl<L, R> Typed for Eq<L, R>
where
    L: Typed,
    R: Typed,
    L::Ty: Type<Output = <R::Ty as Type>::Output>,
{
    type Ty = Bool;
}
