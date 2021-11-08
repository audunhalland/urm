use super::Database;
use crate::builder::{Build, QueryBuilder};
use crate::lower::{Lower, Lowered};
use crate::ty;
use crate::value::{Scalar, Vector};

#[derive(Clone, Debug)]
pub struct Postgres;

impl Database for Postgres {}

impl<T> ty::Typed<Postgres> for Scalar<T>
where
    T: sqlx::Type<sqlx::Postgres> + Send + Sync + 'static,
{
    type Ty = ty::Unit<T>;
}

impl<T> Lower<Postgres> for Scalar<T>
where
    T: sqlx::Type<sqlx::Postgres> + Send + Sync + 'static,
{
    fn lower(self) -> Option<Lowered<Postgres>> {
        Some(Lowered::Expr(Box::new(self)))
    }
}

impl<T> Build<Postgres> for Scalar<T>
where
    T: sqlx::Type<sqlx::Postgres> + Send + Sync + 'static,
{
    fn build(&self, builder: &mut QueryBuilder<Postgres>) {
        builder.push("?");
    }
}

impl<T> ty::Typed<Postgres> for Vector<T>
where
    T: sqlx::Type<sqlx::Postgres> + Send + Sync + 'static,
{
    type Ty = ty::Vector<T>;
}

impl<T> Lower<Postgres> for Vector<T>
where
    T: sqlx::Type<sqlx::Postgres> + Send + Sync + 'static,
{
    fn lower(self) -> Option<Lowered<Postgres>> {
        Some(Lowered::Expr(Box::new(self)))
    }
}

impl<T> Build<Postgres> for Vector<T>
where
    T: sqlx::Type<sqlx::Postgres> + Send + Sync + 'static,
{
    fn build(&self, builder: &mut QueryBuilder<Postgres>) {
        builder.push("?");
    }
}
