//!
//! Primitive projection of columns/fields.
//!

use super::{LocalId, Outcome, ProjectAndProbe, ProjectFrom};
use crate::engine::{Probing, QueryField};
use crate::{Database, Table, UrmResult};

pub struct Primitive<T, Out> {
    name: &'static str,
    local_id: LocalId,
    table: std::marker::PhantomData<T>,
    out: std::marker::PhantomData<Out>,
}

impl<T, Out> Primitive<T, Out>
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

impl<T, Out> ProjectFrom for Primitive<T, Out>
where
    T: Table,
    Out: Sized + Send + Sync + 'static,
{
    type Table = T;
    type Outcome = PrimitiveOutcome<Out>;
}

impl<DB, T, Out> ProjectAndProbe<DB> for Primitive<T, Out>
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
pub struct PrimitiveOutcome<Out> {
    out: std::marker::PhantomData<Out>,
}

impl<Out> Outcome for PrimitiveOutcome<Out>
where
    Out: Send + Sync + 'static,
{
    type Unit = Out;
    type Output = Out;
}
