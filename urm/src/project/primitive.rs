use super::{LocalId, Outcome, ProjectAndProbe, ProjectFrom};
use crate::engine::{Probing, QueryField};
use crate::{Table, UrmResult};

pub trait PrimitiveField: ProjectFrom {
    fn name(&self) -> &'static str;

    fn local_id(&self) -> LocalId;
}

pub struct Primitive<T, O> {
    name: &'static str,
    local_id: LocalId,
    table: std::marker::PhantomData<T>,
    output: std::marker::PhantomData<O>,
}

impl<T, O> Primitive<T, O>
where
    T: Table,
    O: Sized + Send + Sync + 'static,
{
    pub fn new(name: &'static str, local_id: LocalId) -> Self {
        Self {
            name,
            local_id,
            table: std::marker::PhantomData,
            output: std::marker::PhantomData,
        }
    }
}

impl<T, O> ProjectFrom for Primitive<T, O>
where
    T: Table,
    O: Sized + Send + Sync + 'static,
{
    type Table = T;
    type Outcome = PrimitiveOutcome<O>;
}

impl<T, O> PrimitiveField for Primitive<T, O>
where
    T: Table,
    O: Sized + Send + Sync + 'static,
{
    fn name(&self) -> &'static str {
        self.name
    }

    fn local_id(&self) -> LocalId {
        self.local_id
    }
}

///
/// Primitive field type that is just a 'column',
/// not a foreign reference.
///
pub struct PrimitiveOutcome<T> {
    table: std::marker::PhantomData<T>,
}

impl<O> Outcome for PrimitiveOutcome<O>
where
    O: Send + Sync + 'static,
{
    type Unit = O;
    type Output = O;
}

impl<F, V> ProjectAndProbe for F
where
    F: PrimitiveField<Outcome = PrimitiveOutcome<V>>,
{
    fn project_and_probe(self, probing: &Probing) -> UrmResult<()> {
        probing
            .select()
            .projection
            .lock()
            .insert(self.local_id(), QueryField::Primitive);
        Ok(())
    }
}
