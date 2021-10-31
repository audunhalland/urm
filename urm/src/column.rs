//!
//! Primitive projection of columns/fields.
//!

use crate::engine::{Probing, QueryField};
use crate::project::{LocalId, ProjectAndProbe, ProjectFrom};
use crate::ty::Type;
use crate::{Database, Table, UrmResult};

pub struct Column<T, Out> {
    name: &'static str,
    local_id: LocalId,
    table: std::marker::PhantomData<T>,
    out: std::marker::PhantomData<Out>,
}

impl<T, Out> Column<T, Out>
where
    T: Table,
    Out: Sized + Send + Sync + 'static,
{
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn new(name: &'static str, local_id: LocalId) -> Self {
        Self {
            name,
            local_id,
            table: std::marker::PhantomData,
            out: std::marker::PhantomData,
        }
    }
}

impl<T, Out> ProjectFrom for Column<T, Out>
where
    T: Table,
    Out: Sized + Send + Sync + 'static,
{
    type Table = T;
    type Ty = PrimitiveType<Out>;
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

///
/// Primitive outcome is just the value of a 'column', no fancy type mapping.
///
pub struct PrimitiveType<Out> {
    out: std::marker::PhantomData<Out>,
}

impl<Out> Type for PrimitiveType<Out>
where
    Out: Send + Sync + 'static,
{
    type Unit = Out;
    type Output = Out;
}
