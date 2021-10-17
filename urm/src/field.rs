use async_trait::*;

use crate::engine::Projection;
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
    fn probe_with<'c, Func, Out>(
        &self,
        func: Func,
        ctx: &'c ::async_graphql::context::Context<'_>,
    ) -> probe_shim::ProbeShim<'c, Self, Func, Self::Describe, Out>
    where
        Self::Describe: QuantifyProbe<Out>,
        Func: Fn(<Self::Describe as DescribeField>::Value) -> Out,
        Out: crate::Probe,
    {
        probe_shim::ProbeShim::new(ProbeMapping::new(func), ctx)
    }
}

/// Field metadata
pub trait DescribeField: Sized {
    type Value;
    type Output;
}

/// Something that can be probe-projected directly
#[async_trait]
pub trait ProjectAndProbe: Field {
    fn project(&self, projection: &mut Projection);
    async fn probe<'a>(&'a self);
}

pub trait QuantifyProbe<U>: DescribeField + Send + Sync + 'static {
    type Q: Quantify<U>;
}

pub struct Scalar<T> {
    ph: std::marker::PhantomData<T>,
}

impl<T> DescribeField for Scalar<T> {
    type Value = T;
    type Output = T;
}

#[async_trait]
impl<F, T> ProjectAndProbe for F
where
    F: Field<Describe = Scalar<T>>,
{
    fn project(&self, projection: &mut Projection) {
        projection.project_basic_field(F::local_id());
    }

    async fn probe(&self) {}
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

pub struct ProbeMapping<Func, In: QuantifyProbe<Out>, Out>(
    Func,
    std::marker::PhantomData<In>,
    std::marker::PhantomData<Out>,
);

impl<Func, In, Out> ProbeMapping<Func, In, Out>
where
    In: QuantifyProbe<Out>,
{
    fn new(func: Func) -> Self {
        Self(func, std::marker::PhantomData, std::marker::PhantomData)
    }
}

impl<Func, In, Out> DescribeField for ProbeMapping<Func, In, Out>
where
    In: QuantifyProbe<Out>,
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

    pub struct ProbeShim<'c, F: Field, Func, In: QuantifyProbe<Out>, Out: crate::Probe> {
        probe_mapping: ProbeMapping<Func, In, Out>,
        field: std::marker::PhantomData<F>,
        ctx: &'c ::async_graphql::context::Context<'c>,
    }

    impl<'c, F, Func, In, Out> ProbeShim<'c, F, Func, In, Out>
    where
        F: Field,
        In: QuantifyProbe<Out>,
        Out: crate::Probe,
    {
        pub fn new(
            probe_project: ProbeMapping<Func, In, Out>,
            ctx: &'c ::async_graphql::context::Context<'c>,
        ) -> Self {
            Self {
                probe_mapping: probe_project,
                field: std::marker::PhantomData,
                ctx,
            }
        }
    }

    impl<'c, F, Func, In, Out> Field for ProbeShim<'c, F, Func, In, Out>
    where
        F: Field<Describe = In>,
        Func: Send + Sync + 'static,
        In: QuantifyProbe<Out>,
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
    impl<'c, F, Func, In, Out> ProjectAndProbe for ProbeShim<'c, F, Func, In, Out>
    where
        F: Field<Describe = In>,
        Func: Send + Sync + 'static,
        In: QuantifyProbe<Out>,
        Out: crate::Probe + Send + Sync + 'static,
    {
        fn project(&self, projection: &mut Projection) {
            projection.project_basic_field(F::local_id());
        }

        async fn probe(&self) {
            // let next_probe = self.probe_mapping.0();
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
