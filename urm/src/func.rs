//!
//! Type mapping through function application
//!

use crate::builder::{Build, QueryBuilder};
use crate::lower::Lower;
use crate::ty::{Nullable, Type, Typed};
use crate::Database;

/// A unary function, accepting one parameter/operand
pub trait Unary<DB: Database, T>: Send + Sync + 'static {
    type Output: Type;

    fn build(&self, builder: &mut QueryBuilder<DB>, operand: &T);
}

impl<DB, Op, T> Typed<DB> for (Op, T)
where
    DB: Database,
    Op: Unary<DB, T>,
{
    type Ty = Op::Output;
}

impl<DB, Op, T> Lower<DB> for (Op, T)
where
    DB: Database,
    Op: Unary<DB, T>,
    T: Lower<DB> + Build<DB>,
{
    type Target = Self;

    fn lower(self) -> Option<Self> {
        Some(self)
    }
}

impl<DB, Op, T> Build<DB> for (Op, T)
where
    DB: Database,
    Op: Unary<DB, T>,
    T: Build<DB>,
{
    fn build(&self, builder: &mut QueryBuilder<DB>) {
        self.0.build(builder, &self.1);
    }
}

/// A binary function, accepting two parameters/operands
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

impl<DB, Op, L, R> Lower<DB> for (Op, L, R)
where
    DB: Database,
    Op: Binary<DB, L, R>,
    L: Lower<DB> + Build<DB>,
    R: Lower<DB> + Build<DB>,
{
    type Target = Self;

    fn lower(self) -> Option<Self> {
        Some(self)
    }
}

impl<DB, Op, L, R> Build<DB> for (Op, L, R)
where
    DB: Database,
    Op: Binary<DB, L, R>,
    L: Build<DB>,
    R: Build<DB>,
{
    fn build(&self, builder: &mut QueryBuilder<DB>) {
        self.0.build(builder, &self.1, &self.2);
    }
}

/// Binary function that tests equality between two given operands
#[derive(Debug)]
pub struct Equals;

impl<DB, L, R> Binary<DB, L, R> for Equals
where
    DB: Database,
    L: Lower<DB> + Build<DB>,
    R: Lower<DB> + Build<DB>,
    L::Ty: Type<Output = <R::Ty as Type>::Output>,
{
    type Output = Nullable<bool>;

    fn build(&self, builder: &mut QueryBuilder<DB>, left: &L, right: &R) {
        left.build(builder);
        builder.push(" = ");
        right.build(builder);
    }
}
