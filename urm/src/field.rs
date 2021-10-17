use async_trait::*;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::engine::{Projection, QueryEngine};
use crate::{Node, Table};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LocalId(pub u16);

pub trait Field: Sized + Send + Sync {
    type Owner;
    type Describe: DescribeField;

    fn name() -> &'static str;

    fn local_id() -> LocalId;

    /// Make a field probe-able by supplying a mapper
    /// function and probing context
    #[cfg(feature = "async_graphql")]
    fn probe_with<'c, T, Func, Out>(
        &self,
        func: Func,
        ctx: &'c ::async_graphql::context::Context<'_>,
    ) -> probe_shim::ForeignProbeShim<'c, Self, T, Func, Self::Describe, Out>
    where
        T: Table,
        Self::Describe: QuantifyProbe<Out> + DescribeField<Value = Node<T>>,
        Func: Fn(<Self::Describe as DescribeField>::Value) -> Out,
        Out: crate::Probe,
    {
        let mapping = ProbeMapping::new(func);

        probe_shim::ForeignProbeShim::new(mapping, ctx)
    }
}

/// Field metadata
pub trait DescribeField: Sized {
    type Value: Send + Sync + 'static;
    type Output;
}

/// Something that can be probe-projected directly
#[async_trait]
pub trait ProjectAndProbe: Field {
    async fn project_and_probe(
        &self,
        engine: &Arc<Mutex<QueryEngine>>,
        projection: Arc<Mutex<crate::engine::Projection>>,
    );
}

pub trait QuantifyProbe<U>: DescribeField + Send + Sync + 'static {
    type Q: Quantify<U>;
}

pub struct Scalar<T> {
    table: std::marker::PhantomData<T>,
}

impl<T> DescribeField for Scalar<T>
where
    T: Send + Sync + 'static,
{
    type Value = T;
    type Output = T;
}

#[async_trait]
impl<F, T> ProjectAndProbe for F
where
    F: Field<Describe = Scalar<T>>,
{
    async fn project_and_probe(
        &self,
        engine: &Arc<Mutex<QueryEngine>>,
        projection: Arc<Mutex<crate::engine::Projection>>,
    ) {
        projection.lock().project_basic_field(F::local_id());
    }
}

pub struct ForeignOneToOne<T: Table> {
    ph: std::marker::PhantomData<T>,
}

impl<T: Table> DescribeField for ForeignOneToOne<T> {
    type Value = Node<T>;
    type Output = Node<T>;
}

impl<T: Table, U> QuantifyProbe<U> for ForeignOneToOne<T> {
    type Q = Unit;
}

pub struct ForeignOneToMany<T: Table> {
    ph: std::marker::PhantomData<T>,
}

impl<T: Table> DescribeField for ForeignOneToMany<T> {
    type Value = Node<T>;
    type Output = Vec<Node<T>>;
}

impl<T: Table, U> QuantifyProbe<U> for ForeignOneToMany<T> {
    type Q = Vector;
}

pub struct ForeignField {
    pub foreign_table: &'static dyn Table,
}

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

impl<Func, In, Out> DescribeField for ProbeMapping<Func, In, Out>
where
    In: QuantifyProbe<Out>,
    Out: Send + Sync + 'static,
{
    type Value = Out;
    type Output = <In::Q as Quantify<Out>>::Output;
}

pub struct Unit;
pub struct Vector;

pub trait Quantify<T> {
    type Output;
}

impl<T> Quantify<T> for Unit {
    type Output = T;
}

impl<T> Quantify<T> for Vector {
    type Output = Vec<T>;
}

#[cfg(feature = "async_graphql")]
pub mod probe_shim {
    use super::*;

    pub struct ForeignProbeShim<
        'c,
        F: Field,
        T: Table,
        Func,
        DescIn: DescribeField,
        Out: crate::Probe,
    > {
        pub probe_mapping: ProbeMapping<Func, DescIn::Value, Out>,
        field: std::marker::PhantomData<F>,
        table: std::marker::PhantomData<T>,
        ctx: &'c ::async_graphql::context::Context<'c>,
    }

    impl<'c, F, T, Func, DescIn, Out> ForeignProbeShim<'c, F, T, Func, DescIn, Out>
    where
        F: Field,
        T: Table,
        DescIn: DescribeField,
        Out: crate::Probe,
    {
        pub fn new(
            probe_project: ProbeMapping<Func, DescIn::Value, Out>,
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

    impl<'c, F, T, Func, DescIn, Out> Field for ForeignProbeShim<'c, F, T, Func, DescIn, Out>
    where
        F: Field<Describe = DescIn>,
        T: Table,
        Func: Send + Sync + 'static,
        DescIn: QuantifyProbe<Out>,
        Out: crate::Probe + Send + Sync + 'static,
    {
        type Owner = F::Owner;
        type Describe = ProbeMapping<Func, F::Describe, Out>;

        fn name() -> &'static str {
            F::name()
        }

        fn local_id() -> LocalId {
            F::local_id()
        }
    }

    #[async_trait]
    impl<'c, F, T, Func, DescIn, Out> ProjectAndProbe for ForeignProbeShim<'c, F, T, Func, DescIn, Out>
    where
        F: Field<Describe = DescIn>,
        T: Table,
        Func: (Fn(<DescIn as DescribeField>::Value) -> Out) + Send + Sync + 'static,
        DescIn: QuantifyProbe<Out>,
        Out: crate::Probe + Send + Sync + 'static,
    {
        async fn project_and_probe(
            &self,
            engine: &Arc<Mutex<QueryEngine>>,
            projection: Arc<Mutex<crate::engine::Projection>>,
        ) {
            let _sub_projection = Arc::new(Mutex::new(Projection::new()));

            // proj_lock.foreign_subselect(F::local_id(), sub_projection.clone());

            let node = Node::<T>::setup(crate::engine::ProjectionSetup::new(engine.clone()));
            //let probe = self.probe_mapping.0(node);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MyTable;
    struct MyField;

    impl Field for MyField {
        type Owner = MyTable;
        type Describe = Scalar<String>;

        fn name() -> &'static str {
            return "test";
        }

        fn local_id() -> LocalId {
            LocalId(0)
        }
    }
}
