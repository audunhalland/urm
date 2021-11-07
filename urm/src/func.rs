//!
//! Type mapping through function application
//!

use crate::build::Build;
use crate::builder::QueryBuilder;
use crate::ty::{Type, Typed, Unit};
use crate::Database;

pub trait Unary<T> {
    type Output: Type;
}

impl<DB, Op, T> Typed<DB> for (Op, T)
where
    DB: Database,
    Op: Unary<T>,
{
    type Ty = Op::Output;
}

pub trait Binary<DB: Database, L, R>: Send + Sync + 'static {
    type Output: Type;

    fn build(&self, builder: &mut QueryBuilder<DB>, left: &L, right: &R);
}

impl<DB, Op, L, R> Typed<DB> for (Op, L, R)
where
    DB: Database,
    Op: Binary<DB, L, R>,
{
    type Ty = Op::Output;
}

impl<DB, Op, L, R> Build<DB> for (Op, L, R)
where
    DB: Database,
    L: Build<DB>,
    R: Build<DB>,
    Op: Binary<DB, L, R>,
{
    fn build(&self, builder: &mut QueryBuilder<DB>) {
        self.0.build(builder, &self.1, &self.2)
    }
}

#[derive(Debug)]
pub struct Equals<DB>(std::marker::PhantomData<DB>);

impl<DB> Equals<DB> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<DB, L, R> Binary<DB, L, R> for Equals<DB>
where
    DB: Database,
    L: Build<DB>,
    R: Build<DB>,
    L::Ty: Type<Output = <R::Ty as Type>::Output>,
{
    type Output = Unit<bool>;

    fn build(&self, builder: &mut QueryBuilder<DB>, left: &L, right: &R) {
        left.build(builder);
        builder.push(" = ");
        right.build(builder);
    }
}
