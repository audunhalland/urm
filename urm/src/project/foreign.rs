use super::{LocalId, Outcome, ProjectAndProbe, ProjectFrom};
use crate::engine::{Probing, QueryField};
use crate::expr;
use crate::{Filter, Instance, Node, Table, UrmResult};

/// Quantification of some unit value into quantified output
/// for the probing process
pub trait ForeignOutcome<U>: Outcome {
    type Quantify: Quantify<U>;
}

pub struct Foreign<T1, T2, M: Outcome> {
    join_predicate: expr::Predicate,
    predicate: Option<expr::Predicate>,
    source_table: std::marker::PhantomData<T1>,
    foreign_table: std::marker::PhantomData<T2>,
    outcome: std::marker::PhantomData<M>,
}

impl<T1, T2, M> Foreign<T1, T2, M>
where
    T1: Table,
    T2: Table + Instance,
    M: Outcome,
{
    pub fn new(join_predicate: expr::Predicate) -> Self {
        Self {
            join_predicate,
            predicate: None,
            source_table: std::marker::PhantomData,
            foreign_table: std::marker::PhantomData,
            outcome: std::marker::PhantomData,
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
        M: Outcome<Unit = Node<T>> + ForeignOutcome<Out>,
        Func: Fn(<M as Outcome>::Unit) -> Out,
        Out: async_graphql::ContainerType,
    {
        probe_shim::ForeignProbeShim::new(self, ProbeOutcome::new(func), ctx)
    }
}

impl<T1, T2, M> Filter for Foreign<T1, T2, M>
where
    T2: Table + Instance,
    M: Outcome,
{
    type Table = T2;

    fn range<R>(self, _r: R) -> Self
    where
        R: crate::filter::Range,
    {
        self
    }
}

impl<T1, T2, M> ProjectFrom for Foreign<T1, T2, M>
where
    T1: Table,
    T2: Table,
    M: Outcome,
{
    type Table = T1;
    type Outcome = M;
}

/// A 'foreign' reference field that points to
/// at most one foreign entity
pub struct OneToOne<T: Table> {
    foreign: std::marker::PhantomData<T>,
}

impl<T: Table> Outcome for OneToOne<T> {
    type Unit = Node<T>;
    type Output = Node<T>;
}

impl<T: Table, U> ForeignOutcome<U> for OneToOne<T> {
    type Quantify = Unit;
}

/// A 'foreign' reference field that points to
/// potentially many foreign entities.
pub struct OneToMany<T: Table> {
    foreign: std::marker::PhantomData<T>,
}

impl<T: Table> Outcome for OneToMany<T> {
    type Unit = Node<T>;
    type Output = Vec<Node<T>>;
}

impl<T: Table, U> ForeignOutcome<U> for OneToMany<T> {
    type Quantify = Vector;
}

/// Function wrapper to map from some table node into Probe
pub struct ProbeOutcome<Func, In, Out> {
    func: Func,
    in_ph: std::marker::PhantomData<In>,
    out_ph: std::marker::PhantomData<Out>,
}

impl<Func, In, Out> ProbeOutcome<Func, In, Out>
where
    Func: Fn(In) -> Out,
{
    fn new(func: Func) -> Self {
        Self {
            func,
            in_ph: std::marker::PhantomData,
            out_ph: std::marker::PhantomData,
        }
    }
}

impl<Func, In, Out> Outcome for ProbeOutcome<Func, In, Out>
where
    Func: Send + Sync + 'static,
    In: ForeignOutcome<Out>,
    Out: Send + Sync + 'static,
    <<In as ForeignOutcome<Out>>::Quantify as Quantify<Out>>::Output: Send + Sync + 'static,
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

    pub struct ForeignProbeShim<'c, T1, T2, M: Outcome, Func, Out: async_graphql::ContainerType> {
        field: Foreign<T1, T2, M>,
        probe_mech: ProbeOutcome<Func, M::Unit, Out>,
        ctx: &'c ::async_graphql::context::Context<'c>,
    }

    impl<'c, T1, T2, M, Func, Out> ForeignProbeShim<'c, T1, T2, M, Func, Out>
    where
        M: Outcome,
        Out: async_graphql::ContainerType,
    {
        pub fn new(
            field: Foreign<T1, T2, M>,
            probe_mech: ProbeOutcome<Func, M::Unit, Out>,
            ctx: &'c ::async_graphql::context::Context<'c>,
        ) -> Self {
            Self {
                field,
                probe_mech,
                ctx,
            }
        }
    }

    impl<'c, T1, T2, M, Func, Out> ProjectFrom for ForeignProbeShim<'c, T1, T2, M, Func, Out>
    where
        T1: Table,
        T2: Table + Instance,
        M: ForeignOutcome<Out>,
        Func: Send + Sync + 'static,
        <<M as ForeignOutcome<Out>>::Quantify as Quantify<Out>>::Output: Send + Sync + 'static,
        Out: async_graphql::ContainerType + Send + Sync + 'static,
    {
        type Table = T1;
        type Outcome = ProbeOutcome<Func, M, Out>;
    }

    impl<'c, T1, T2, M, Func, Out> ProjectAndProbe for ForeignProbeShim<'c, T1, T2, M, Func, Out>
    where
        T1: Table,
        T2: Table + Instance,
        M: Outcome<Unit = Node<T2>> + ForeignOutcome<Out>,
        Func: (Fn(<M as Outcome>::Unit) -> Out) + Send + Sync + 'static,
        <<M as ForeignOutcome<Out>>::Quantify as Quantify<Out>>::Output: Send + Sync + 'static,
        Out: async_graphql::ContainerType + Send + Sync + 'static,
    {
        fn project_and_probe(self, probing: &Probing) -> UrmResult<()> {
            let foreign_table = T2::instance();
            let sub_select = probing
                .engine()
                .query
                .lock()
                .new_select(foreign_table, self.field.predicate);

            {
                let mut proj_lock = probing.select().projection.lock();
                let join_predicate = self.field.join_predicate;

                proj_lock.insert(
                    // FIXME: This should be "dynamic" in some way,
                    // or use a different type of key when projecting.
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

            let container = (self.probe_mech.func)(sub_node);
            crate::probe::probe_container(&container, self.ctx);

            Ok(())
        }
    }
}
