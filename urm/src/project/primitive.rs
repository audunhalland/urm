use super::{LocalId, Outcome, ProjectAndProbe, ProjectFrom};
use crate::engine::{Probing, QueryField};
use crate::{Table, UrmResult};

pub trait PrimitiveField: ProjectFrom {
    fn name(&self) -> &'static str;

    fn local_id(&self) -> LocalId;
}

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

impl<T, Out> PrimitiveField for Primitive<T, Out>
where
    T: Table,
    Out: Sized + Send + Sync + 'static,
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

impl<F, Out> ProjectAndProbe for F
where
    F: PrimitiveField<Outcome = PrimitiveOutcome<Out>>,
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
