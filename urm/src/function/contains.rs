use crate::builder::{Build, QueryBuilder};
use crate::database::postgres::Postgres;
use crate::database::Database;
use crate::lower::{Lower, Lowered};
use crate::ty::{Nullable, ScalarTyped, Type, Typed};

/// Binary function that tests equality between two given operands
pub struct Contains<L, R>(pub L, pub R);

impl<DB, C, I> Typed<DB> for Contains<C, I>
where
    DB: Database,
    C: Lower<DB> + Build<DB>,
    I: Lower<DB> + Build<DB> + ScalarTyped<DB, <C::Ty as Type>::Unit>,
{
    type Ty = Nullable<bool>;
}

#[cfg(feature = "postgres")]
impl<C, I> Lower<Postgres> for Contains<C, I>
where
    C: Lower<Postgres> + Build<Postgres>,
    I: Lower<Postgres> + Build<Postgres> + ScalarTyped<Postgres, <C::Ty as Type>::Unit>,
{
    fn lower(self) -> Option<Lowered<Postgres>> {
        Some(Lowered::Expr(Box::new(self)))
    }
}

#[cfg(feature = "postgres")]
impl<C, I> Build<Postgres> for Contains<C, I>
where
    C: Lower<Postgres> + Build<Postgres>,
    I: Lower<Postgres> + Build<Postgres> + ScalarTyped<Postgres, <C::Ty as Type>::Unit>,
{
    fn build(&self, builder: &mut QueryBuilder<Postgres>) {
        self.1.build(builder);
        builder.push(" = any(");
        self.0.build(builder);
        builder.push(")");
    }
}
