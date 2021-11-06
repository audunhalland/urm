//!
//! Type mapping through function application
//!

use crate::builder::QueryBuilder;
use crate::ty::{Bool, Type, Typed};
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

pub trait Binary<DB: Database, L, R> {
    type Output: Type;

    fn build_query(&self, builder: &mut QueryBuilder<DB>, left: &L, right: &R);
}

impl<DB, Op, L, R> Typed<DB> for (Op, L, R)
where
    DB: Database,
    Op: Binary<DB, L, R>,
{
    type Ty = Op::Output;
}

#[derive(Debug)]
pub struct Equals<DB, L, R>(std::marker::PhantomData<(DB, L, R)>);

impl<DB, L, R> Binary<DB, L, R> for Equals<DB, L, R>
where
    DB: Database,
    L: Typed<DB>,
    R: Typed<DB>,
    L::Ty: Type<Output = <R::Ty as Type>::Output>,
{
    type Output = Bool;

    fn build_query(&self, builder: &mut QueryBuilder<DB>, left: &L, right: &R) {
        // self.0.build_expr(builder);
        builder.push(" = ");
        // self.1.build_expr(builder);
    }
}
