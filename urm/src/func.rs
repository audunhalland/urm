//!
//! Type mapping through function application
//!

use crate::ty::{Bool, Type, Typed};

pub trait Unary<T> {
    type Output: Type;
}

impl<Op, T> Typed for (Op, T)
where
    Op: Unary<T>,
{
    type Ty = Op::Output;
}

pub trait Binary<L, R> {
    type Output: Type;
}

impl<Op, L, R> Typed for (Op, L, R)
where
    Op: Binary<L, R>,
{
    type Ty = Op::Output;
}

#[derive(Debug)]
pub struct Equals<L, R>(std::marker::PhantomData<(L, R)>);

impl<L, R> Binary<L, R> for Equals<L, R>
where
    L: Typed,
    R: Typed,
    L::Ty: Type<Output = <R::Ty as Type>::Output>,
{
    type Output = Bool;
}
