use crate::engine::Projection;
use crate::{Node, Table};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LocalId(pub u16);

pub trait Field: Sized + Send + Sync + 'static {
    type Owner;
    type Describe: DescribeField;

    fn name() -> &'static str;

    fn local_id() -> LocalId;

    /// Map the field value into another type
    #[cfg(feature = "async_graphql")]
    fn probe_as<U, Func>(
        &self,
        func: Func,
        ctx: &::async_graphql::context::Context<'_>,
    ) -> probe_shim::ProbeShim<Self, U, Func, Self::Describe>
    where
        Self::Describe: ProbeAs<U>,
        Func: Fn(<Self::Describe as DescribeField>::Value) -> U,
    {
        probe_shim::ProbeShim::new(ProbeProject::new(func))
    }
}

/// Field metadata
pub trait DescribeField: Sized {
    type Value;
    type Output;
}

/// Something that can be projected directly
pub trait ProjectField: Field {
    fn project(self, projection: &mut Projection);
}

pub trait ProbeAs<U>: DescribeField + Send + Sync + 'static {
    type Q: Quantify<U>;
}

pub struct Scalar<T> {
    ph: std::marker::PhantomData<T>,
}

impl<T> DescribeField for Scalar<T> {
    type Value = T;
    type Output = T;
}

impl<F, T> ProjectField for F
where
    F: Field<Describe = Scalar<T>>,
{
    fn project(self, projection: &mut Projection) {
        projection.project_basic_field(F::local_id());
    }
}

pub struct ForeignOneToOne<T: Table> {
    ph: std::marker::PhantomData<T>,
}

impl<T: Table> DescribeField for ForeignOneToOne<T> {
    type Value = Node<T>;
    type Output = Node<T>;
}

impl<T: Table, U> ProbeAs<U> for ForeignOneToOne<T> {
    type Q = Unit;
}

pub struct ForeignOneToMany<T: Table> {
    ph: std::marker::PhantomData<T>,
}

impl<T: Table> DescribeField for ForeignOneToMany<T> {
    type Value = Node<T>;
    type Output = Vec<Node<T>>;
}

impl<T: Table, U> ProbeAs<U> for ForeignOneToMany<T> {
    type Q = Vector;
}

pub struct ForeignField {
    pub foreign_table: &'static dyn Table,
}

pub struct ProbeProject<T, Func, M: ProbeAs<T>>(
    Func,
    std::marker::PhantomData<T>,
    std::marker::PhantomData<M>,
);

impl<T, Func, M> ProbeProject<T, Func, M>
where
    M: ProbeAs<T>,
{
    fn new(func: Func) -> Self {
        Self(func, std::marker::PhantomData, std::marker::PhantomData)
    }
}

impl<T, Func, M> DescribeField for ProbeProject<T, Func, M>
where
    M: ProbeAs<T>,
{
    type Value = T;
    type Output = <M::Q as Quantify<T>>::Output;
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

    pub struct ProbeShim<F: Field, U, Func, P: ProbeAs<U>> {
        probe_project: ProbeProject<U, Func, P>,
        field: std::marker::PhantomData<F>,
    }

    impl<F: Field, U, Func, P: ProbeAs<U>> ProbeShim<F, U, Func, P> {
        pub fn new(probe_project: ProbeProject<U, Func, P>) -> Self {
            Self {
                probe_project,
                field: std::marker::PhantomData,
            }
        }
    }

    impl<F, U, Func, P> Field for ProbeShim<F, U, Func, P>
    where
        F: Field<Describe = P>,
        U: Send + Sync + 'static,
        Func: Send + Sync + 'static,
        P: ProbeAs<U>,
    {
        type Owner = F::Owner;
        type Describe = ProbeProject<U, Func, F::Describe>;

        fn name() -> &'static str {
            F::name()
        }

        fn local_id() -> LocalId {
            F::local_id()
        }
    }

    impl<F, U, Func, P> ProjectField for ProbeShim<F, U, Func, P>
    where
        F: Field<Describe = P>,
        U: Send + Sync + 'static,
        Func: Send + Sync + 'static,
        P: ProbeAs<U>,
    {
        fn project(self, projection: &mut Projection) {
            projection.project_basic_field(F::local_id());
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
