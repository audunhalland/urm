//!
//! Type mapping through function application
//!

use crate::builder::{Build, QueryBuilder};
use crate::lower::{Lower, Lowered};
use crate::ty::{Nullable, Type, Typed};
use crate::Database;

/// Binary function that tests equality between two given operands
pub struct Equals<L, R>(pub L, pub R);

impl<DB, L, R> Typed<DB> for Equals<L, R>
where
    DB: Database,
    L: Lower<DB> + Build<DB>,
    R: Lower<DB> + Build<DB>,
    L::Ty: Type<Output = <R::Ty as Type>::Output>,
{
    type Ty = Nullable<bool>;
}

impl<DB, L, R> Lower<DB> for Equals<L, R>
where
    DB: Database,
    L: Lower<DB> + Build<DB>,
    R: Lower<DB> + Build<DB>,
    L::Ty: Type<Output = <R::Ty as Type>::Output>,
{
    fn lower(self) -> Option<Lowered<DB>> {
        Some(Lowered::Expr(Box::new(self)))
    }
}

impl<DB, L, R> Build<DB> for Equals<L, R>
where
    DB: Database,
    L: Lower<DB> + Build<DB>,
    R: Lower<DB> + Build<DB>,
    L::Ty: Type<Output = <R::Ty as Type>::Output>,
{
    fn build(&self, builder: &mut QueryBuilder<DB>) {
        self.0.build(builder);
        builder.push(" = ");
        self.1.build(builder);
    }
}
