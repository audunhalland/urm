//!
//! Primitive projection of columns/fields.
//!

use crate::engine::{Probing, QueryField};
use crate::project::{LocalId, ProjectAndProbe, ProjectFrom};
use crate::ty::{Type, Typed};
use crate::{Database, Table, UrmResult};

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

impl<T, Ty> Typed for Column<T, Ty>
where
    T: Table,
    Ty: Type,
{
    type Ty = Ty;
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
