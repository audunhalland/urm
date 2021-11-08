use crate::builder::Build;
use crate::builder::QueryBuilder;
use crate::func::Binary;
use crate::lower::Lower;
use crate::ty::{Nullable, ScalarTyped};
use crate::Database;

#[derive(Debug)]
pub struct And;

impl<DB, L, R> Binary<DB, L, R> for And
where
    DB: Database,
    L: Lower<DB> + Build<DB> + ScalarTyped<DB, bool>,
    R: Lower<DB> + Build<DB> + ScalarTyped<DB, bool>,
{
    type Output = Nullable<bool>;

    fn build(&self, builder: &mut QueryBuilder<DB>, left: &L, right: &R) {
        builder.push("(");
        left.build(builder);
        builder.push(" AND ");
        right.build(builder);
        builder.push(")");
    }
}
