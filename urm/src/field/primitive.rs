use super::{Field, FieldMechanics, LocalId, ProjectAndProbe};
use crate::engine::{Probing, QueryField};
use crate::{Table, UrmResult};

pub trait PrimitiveField: Field {
    fn name(&self) -> &'static str;

    fn local_id(&self) -> LocalId;
}

pub struct Primitive<T: Table, V: Sized + Send + Sync + 'static> {
    name: &'static str,
    local_id: LocalId,
    table: std::marker::PhantomData<T>,
    value: std::marker::PhantomData<V>,
}

impl<T, V> Primitive<T, V>
where
    T: Table,
    V: Sized + Send + Sync + 'static,
{
    pub fn new(name: &'static str, local_id: LocalId) -> Self {
        Self {
            name,
            local_id,
            table: std::marker::PhantomData,
            value: std::marker::PhantomData,
        }
    }
}

impl<T, V> Field for Primitive<T, V>
where
    T: Table,
    V: Sized + Send + Sync + 'static,
{
    type Table = T;
    type Mechanics = PrimitiveMechanics<V>;
}

impl<T, V> PrimitiveField for Primitive<T, V>
where
    T: Table,
    V: Sized + Send + Sync + 'static,
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
pub struct PrimitiveMechanics<T> {
    table: std::marker::PhantomData<T>,
}

impl<V> FieldMechanics for PrimitiveMechanics<V>
where
    V: Send + Sync + 'static,
{
    type Unit = V;
    type Output = V;
}

impl<F, V> ProjectAndProbe for F
where
    F: PrimitiveField<Mechanics = PrimitiveMechanics<V>>,
{
    fn project_and_probe(&self, probing: &Probing) -> UrmResult<()> {
        probing
            .select()
            .projection
            .lock()
            .insert(self.local_id(), QueryField::Primitive);
        Ok(())
    }
}
