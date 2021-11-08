//!
//! Primitive projection of columns/fields.
//!

use crate::builder::{Build, QueryBuilder};
use crate::database::Database;
use crate::engine::{Probing, QueryField};
use crate::lower::{Lower, Lowered};
use crate::project::{LocalId, ProjectAndProbe, ProjectFrom};
use crate::ty::{Type, Typed};
use crate::{Instance, Table, UrmResult};

pub struct Column<T, Ty> {
    name: &'static str,
    local_id: LocalId,
    table: std::marker::PhantomData<T>,
    ty: std::marker::PhantomData<Ty>,
}

impl<T, Ty> Column<T, Ty>
where
    T: Table,
    Ty: Sized + Send + Sync + 'static,
{
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn new(name: &'static str, local_id: LocalId) -> Self {
        Self {
            name,
            local_id,
            table: std::marker::PhantomData,
            ty: std::marker::PhantomData,
        }
    }
}

impl<T, Ty> Typed<T::DB> for Column<T, Ty>
where
    T: Table,
    Ty: Type,
{
    type Ty = Ty;
}

impl<T, Ty> Lower<T::DB> for Column<T, Ty>
where
    T: Table + Instance,
    Ty: Type,
{
    fn lower(self) -> Option<Lowered<T::DB>> {
        Some(Lowered::Expr(Box::new(self)))
    }
}

impl<T, Ty> Build<T::DB> for Column<T, Ty>
where
    T: Table + Instance,
    Ty: Type,
{
    fn build(&self, builder: &mut QueryBuilder<T::DB>) {
        builder.push(T::instance().name());
        builder.push(".");
        builder.push(self.name);
    }
}

impl<T, Ty> ProjectFrom for Column<T, Ty>
where
    T: Table,
    Ty: Type,
{
    type Table = T;
}

impl<DB, T, Out> ProjectAndProbe<DB> for Column<T, Out>
where
    DB: Database,
    T: Table,
    Out: Sized + Send + Sync + 'static,
{
    fn project_and_probe(self, probing: &Probing<DB>) -> UrmResult<()> {
        probing
            .select()
            .projection
            .lock()
            .insert(self.local_id, QueryField::Primitive);
        Ok(())
    }
}
