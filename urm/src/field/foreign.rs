use super::{Field, FieldMechanics, LocalId, ProjectAndProbe};
use crate::engine::{Probing, QueryField};
use crate::expr;
use crate::{Instance, Node, Table, UrmResult};

pub trait ForeignField: Field {
    type ForeignTable: Table + Instance;

    fn filter<F>(self, _filter: F) -> Self
    where
        F: crate::filter::Filter<Self::ForeignTable>,
    {
        self
    }
}

/// Quantification of some unit value into quantified output
/// for the probing process
pub trait ForeignMechanics<U>: FieldMechanics {
    type Quantify: Quantify<U>;
}

pub struct Foreign<T1, T2, M: FieldMechanics> {
    join_predicate: expr::Predicate,
    source_table: std::marker::PhantomData<T1>,
    foreign_table: std::marker::PhantomData<T2>,
    mechanics: std::marker::PhantomData<M>,
}

impl<T1, T2, M> Foreign<T1, T2, M>
where
    T1: Table,
    T2: Table + Instance,
    M: FieldMechanics,
{
    pub fn new(join_predicate: expr::Predicate) -> Self {
        Self {
            join_predicate,
            source_table: std::marker::PhantomData,
            foreign_table: std::marker::PhantomData,
            mechanics: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "async_graphql")]
    pub fn probe_with<'c, T, Func, Out>(
        self,
        func: Func,
        ctx: &'c ::async_graphql::context::Context<'_>,
    ) -> probe_shim::ForeignProbeShim<'c, T1, T2, M, Func, Out>
    where
        T: Table,
        M: FieldMechanics<Unit = Node<T>> + ForeignMechanics<Out>,
        Func: Fn(<M as FieldMechanics>::Unit) -> Out,
        Out: async_graphql::ContainerType,
    {
        probe_shim::ForeignProbeShim::new(self, ProbeMechanics::new(func), ctx)
    }
}

impl<T1, T2, M> Field for Foreign<T1, T2, M>
where
    T1: Table,
    T2: Table,
    M: FieldMechanics,
{
    type Table = T1;
    type Mechanics = M;
}

impl<T1, T2, M> ForeignField for Foreign<T1, T2, M>
where
    T1: Table,
    T2: Table + Instance,
    M: FieldMechanics,
{
    type ForeignTable = T2;
}

/// A 'foreign' reference field that points to
/// at most one foreign entity
pub struct OneToOneMechanics<T: Table> {
    foreign: std::marker::PhantomData<T>,
}

impl<T: Table> FieldMechanics for OneToOneMechanics<T> {
    type Unit = Node<T>;
    type Output = Node<T>;
}

impl<T: Table, U> ForeignMechanics<U> for OneToOneMechanics<T> {
    type Quantify = Unit;
}

/// A 'foreign' reference field that points to
/// potentially many foreign entities.
pub struct OneToManyMechanics<T: Table> {
    foreign: std::marker::PhantomData<T>,
}

impl<T: Table> FieldMechanics for OneToManyMechanics<T> {
    type Unit = Node<T>;
    type Output = Vec<Node<T>>;
}

impl<T: Table, U> ForeignMechanics<U> for OneToManyMechanics<T> {
    type Quantify = Vector;
}

/// Function wrapper to map from some table node into Probe
pub struct ProbeMechanics<Func, In, Out>(
    Func,
    std::marker::PhantomData<In>,
    std::marker::PhantomData<Out>,
);

impl<Func, In, Out> ProbeMechanics<Func, In, Out>
where
    Func: Fn(In) -> Out,
{
    fn new(func: Func) -> Self {
        Self(func, std::marker::PhantomData, std::marker::PhantomData)
    }
}

impl<Func, In, Out> FieldMechanics for ProbeMechanics<Func, In, Out>
where
    Func: Send + Sync + 'static,
    In: ForeignMechanics<Out>,
    Out: Send + Sync + 'static,
    <<In as ForeignMechanics<Out>>::Quantify as Quantify<Out>>::Output: Send + Sync + 'static,
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
        T1,
        T2,
        M: FieldMechanics,
        Func,
        Out: async_graphql::ContainerType,
    > {
        field: Foreign<T1, T2, M>,
        probe_mapping: ProbeMechanics<Func, M::Unit, Out>,
        ctx: &'c ::async_graphql::context::Context<'c>,
    }

    impl<'c, T1, T2, M, Func, Out> ForeignProbeShim<'c, T1, T2, M, Func, Out>
    where
        M: FieldMechanics,
        Out: async_graphql::ContainerType,
    {
        pub fn new(
            field: Foreign<T1, T2, M>,
            probe_mapping: ProbeMechanics<Func, M::Unit, Out>,
            ctx: &'c ::async_graphql::context::Context<'c>,
        ) -> Self {
            Self {
                field,
                probe_mapping,
                ctx,
            }
        }
    }

    impl<'c, T1, T2, M, Func, Out> Field for ForeignProbeShim<'c, T1, T2, M, Func, Out>
    where
        T1: Table,
        T2: Table + Instance,
        M: ForeignMechanics<Out>,
        Func: Send + Sync + 'static,
        <<M as ForeignMechanics<Out>>::Quantify as Quantify<Out>>::Output: Send + Sync + 'static,
        Out: async_graphql::ContainerType + Send + Sync + 'static,
    {
        type Table = T1;
        type Mechanics = ProbeMechanics<Func, M, Out>;
    }

    impl<'c, T1, T2, M, Func, Out> ProjectAndProbe for ForeignProbeShim<'c, T1, T2, M, Func, Out>
    where
        T1: Table,
        T2: Table + Instance,
        Func: (Fn(<M as FieldMechanics>::Unit) -> Out) + Send + Sync + 'static,
        M: FieldMechanics<Unit = Node<T2>> + ForeignMechanics<Out>,
        <<M as ForeignMechanics<Out>>::Quantify as Quantify<Out>>::Output: Send + Sync + 'static,
        Out: async_graphql::ContainerType + Send + Sync + 'static,
    {
        fn project_and_probe(self, probing: &Probing) -> UrmResult<()> {
            let foreign_table = T2::instance();
            let sub_select = probing.engine().query.lock().new_select(foreign_table);

            {
                let mut proj_lock = probing.select().projection.lock();
                let join_predicate = self.field.join_predicate;

                proj_lock.insert(
                    // FIXME: This should be "dynamic" in some way,
                    // or use a different type of key when projection.
                    // perhaps keyed by the predicates..?
                    LocalId(0),
                    QueryField::Foreign {
                        select: sub_select.clone(),
                        join_predicate,
                    },
                );
            }

            let sub_node = Node::<T2>::new_probe(crate::engine::Probing::new(
                probing.engine().clone(),
                sub_select,
            ));

            let sub_probe = self.probe_mapping.0(sub_node);
            crate::probe::probe_container(&sub_probe, self.ctx);

            Ok(())
        }
    }
}
