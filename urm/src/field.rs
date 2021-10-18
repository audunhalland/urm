use parking_lot::Mutex;
use std::sync::Arc;

use crate::engine::Engine;
use crate::{Instance, Node, Table, UrmResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LocalId(pub u16);

///
/// A field having some data type, that can be found in some database.
///
pub trait Field: Sized + Send + Sync {
    /// The table that owns this field
    type Table;

    /// The field 'mechanics', which determines how the field
    /// behaves in the API
    type Mechanics: FieldMechanics;

    fn name() -> &'static str;

    fn local_id() -> LocalId;

    /// Make a field probe-able by supplying a mapper
    /// function and probing context
    #[cfg(feature = "async_graphql")]
    fn probe_with<'c, T, Func, Out>(
        &self,
        func: Func,
        ctx: &'c ::async_graphql::context::Context<'_>,
    ) -> probe_shim::ForeignProbeShim<'c, Self, T, Func, Self::Mechanics, Out>
    where
        T: Table,
        Self::Mechanics: FieldMechanics<Unit = Node<T>> + QuantifyProbe<Out>,
        Func: Fn(<Self::Mechanics as FieldMechanics>::Unit) -> Out,
        Out: async_graphql::ContainerType,
    {
        probe_shim::ForeignProbeShim::new(ProbeMapping::new(func), ctx)
    }
}

/// Field mechanics
pub trait FieldMechanics: Sized {
    /// Unit of this field type, in case Output is quantified
    type Unit: Send + Sync + 'static;

    /// Final, quantified value of the field (possibly Vec<Self::Unit>).
    type Output: Send + Sync + 'static;
}

/// Something that can be probe-projected directly
pub trait ProjectAndProbe: Field {
    fn project_and_probe(
        &self,
        engine: &Engine,
        projection: &Arc<Mutex<crate::engine::Projection>>,
    ) -> UrmResult<()>;
}

/// Quantification of some unit value into quantified output
/// for the probing process
pub trait QuantifyProbe<U>: FieldMechanics + Send + Sync + 'static {
    type Quantify: Quantify<U>;
}

///
/// Primitive field type that is just a 'column',
/// not a foreign reference.
///
pub struct Primitive<T> {
    table: std::marker::PhantomData<T>,
}

impl<V> FieldMechanics for Primitive<V>
where
    V: Send + Sync + 'static,
{
    type Unit = V;
    type Output = V;
}

impl<F, V> ProjectAndProbe for F
where
    F: Field<Mechanics = Primitive<V>>,
{
    fn project_and_probe(
        &self,
        _engine: &Engine,
        projection: &Arc<Mutex<crate::engine::Projection>>,
    ) -> UrmResult<()> {
        projection.lock().project_primitive_field(F::local_id());
        Ok(())
    }
}

/// A 'foreign' reference field that points to
/// at most one foreign entity
pub struct ForeignOneToOne<T: Table> {
    foreign: std::marker::PhantomData<T>,
}

impl<T: Table> FieldMechanics for ForeignOneToOne<T> {
    type Unit = Node<T>;
    type Output = Node<T>;
}

impl<T: Table, U> QuantifyProbe<U> for ForeignOneToOne<T> {
    type Quantify = Unit;
}

/// A 'foreign' reference field that points to
/// potentially many foreign entities.
pub struct ForeignOneToMany<T: Table> {
    foreign: std::marker::PhantomData<T>,
}

impl<T: Table> FieldMechanics for ForeignOneToMany<T> {
    type Unit = Node<T>;
    type Output = Vec<Node<T>>;
}

impl<T: Table, U> QuantifyProbe<U> for ForeignOneToMany<T> {
    type Quantify = Vector;
}

/// Function wrapper to map from some table node into Probe
pub struct ProbeMapping<Func, In, Out>(
    Func,
    std::marker::PhantomData<In>,
    std::marker::PhantomData<Out>,
);

impl<Func, In, Out> ProbeMapping<Func, In, Out>
where
    Func: Fn(In) -> Out,
{
    fn new(func: Func) -> Self {
        Self(func, std::marker::PhantomData, std::marker::PhantomData)
    }
}

impl<Func, In, Out> FieldMechanics for ProbeMapping<Func, In, Out>
where
    In: QuantifyProbe<Out>,
    Out: Send + Sync + 'static,
    <<In as QuantifyProbe<Out>>::Quantify as Quantify<Out>>::Output: Send + Sync + 'static,
{
    type Unit = Out;
    type Output = <In::Quantify as Quantify<Out>>::Output;
}

/// Quantify some unit.
pub trait Quantify<U> {
    type Output;
}

pub struct Unit;
pub struct Vector;

impl<U> Quantify<U> for Unit {
    type Output = U;
}

impl<U> Quantify<U> for Vector {
    type Output = Vec<U>;
}

#[cfg(feature = "async_graphql")]
pub mod probe_shim {
    use super::*;

    pub struct ForeignProbeShim<
        'c,
        F: Field,
        T: Table,
        Func,
        InType: FieldMechanics,
        Out: async_graphql::ContainerType,
    > {
        pub probe_mapping: ProbeMapping<Func, InType::Unit, Out>,
        field: std::marker::PhantomData<F>,
        table: std::marker::PhantomData<T>,
        ctx: &'c ::async_graphql::context::Context<'c>,
    }

    impl<'c, F, T, Func, InType, Out> ForeignProbeShim<'c, F, T, Func, InType, Out>
    where
        F: Field,
        T: Table,
        InType: FieldMechanics,
        Out: async_graphql::ContainerType,
    {
        pub fn new(
            probe_project: ProbeMapping<Func, InType::Unit, Out>,
            ctx: &'c ::async_graphql::context::Context<'c>,
        ) -> Self {
            Self {
                probe_mapping: probe_project,
                field: std::marker::PhantomData,
                table: std::marker::PhantomData,
                ctx,
            }
        }
    }

    impl<'c, F, T, Func, InType, Out> Field for ForeignProbeShim<'c, F, T, Func, InType, Out>
    where
        F: Field<Mechanics = InType>,
        T: Table,
        Func: Send + Sync + 'static,
        InType: QuantifyProbe<Out>,
        <<InType as QuantifyProbe<Out>>::Quantify as Quantify<Out>>::Output: Send + Sync + 'static,
        Out: async_graphql::ContainerType + Send + Sync + 'static,
    {
        // This is still the source table, not the target table (T)
        type Table = F::Table;

        type Mechanics = ProbeMapping<Func, F::Mechanics, Out>;

        fn name() -> &'static str {
            F::name()
        }

        fn local_id() -> LocalId {
            F::local_id()
        }
    }

    impl<'c, F, T, Func, InType, Out> ProjectAndProbe for ForeignProbeShim<'c, F, T, Func, InType, Out>
    where
        F: Field<Mechanics = InType>,
        T: Table + Instance,
        Func: (Fn(<InType as FieldMechanics>::Unit) -> Out) + Send + Sync + 'static,
        InType: FieldMechanics<Unit = Node<T>> + QuantifyProbe<Out>,
        <<InType as QuantifyProbe<Out>>::Quantify as Quantify<Out>>::Output: Send + Sync + 'static,
        Out: async_graphql::ContainerType + async_graphql::OutputType + Send + Sync + 'static,
    {
        fn project_and_probe(
            &self,
            engine: &Engine,
            projection: &Arc<Mutex<crate::engine::Projection>>,
        ) -> UrmResult<()> {
            let sub_probing = crate::engine::Probing::new(engine.clone());
            let sub_projection = sub_probing.projection().clone();
            let sub_node = Node::<T>::new_probe(sub_probing);

            {
                let mut proj_lock = projection.lock();
                proj_lock.foreign_subselect(F::local_id(), T::instance(), sub_projection);
            }

            let sub_probe = self.probe_mapping.0(sub_node);

            crate::probe::probe_container_field(&sub_probe, self.ctx);

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MyTable;
    struct MyField;

    impl Field for MyField {
        type Table = MyTable;
        type Mechanics = Primitive<String>;

        fn name() -> &'static str {
            return "test";
        }

        fn local_id() -> LocalId {
            LocalId(0)
        }
    }
}
