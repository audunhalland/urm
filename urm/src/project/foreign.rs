use super::{LocalId, Outcome, ProjectAndProbe, ProjectFrom};
use crate::engine::{Probing, QueryField};
use crate::expr;
use crate::{Filter, Instance, Node, Table, UrmResult};

/// Quantification of some unit value into quantified output
/// for the probing process.
pub trait ForeignOutcome<U>: Outcome {
    type Quantify: Quantify<U>;
}

///
/// A projection using a _foreign key_, leading into a foreign table.
///
///
///
pub struct Foreign<T1, T2, O: Outcome> {
    join_predicate: expr::Predicate,
    predicate: Option<expr::Predicate>,
    source_table: std::marker::PhantomData<T1>,
    foreign_table: std::marker::PhantomData<T2>,
    outcome: std::marker::PhantomData<O>,
}

impl<T1, T2, O> Foreign<T1, T2, O>
where
    T1: Table,
    T2: Table + Instance,
    O: Outcome,
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
    pub fn probe_with<'c, T, Func, OutUnit>(
        self,
        func: Func,
        ctx: &'c ::async_graphql::context::Context<'_>,
    ) -> probe_shim::ForeignProbeShim<'c, T1, T2, O, Func, OutUnit>
    where
        T: Table,
        O: Outcome<Unit = Node<T>> + ForeignOutcome<OutUnit>,
        Func: Fn(<O as Outcome>::Unit) -> OutUnit,
        OutUnit: async_graphql::ContainerType,
    {
        probe_shim::ForeignProbeShim::new(self, ProbeOutcome::new(func), ctx)
    }
}

impl<T1, T2, O> Filter for Foreign<T1, T2, O>
where
    T2: Table + Instance,
    O: Outcome,
{
    type Table = T2;

    fn range<R>(self, _r: R) -> Self
    where
        R: crate::filter::Range,
    {
        self
    }
}

impl<T1, T2, O> ProjectFrom for Foreign<T1, T2, O>
where
    T1: Table,
    T2: Table,
    O: Outcome,
{
    type Table = T1;
    type Outcome = O;
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
pub struct ProbeOutcome<Func, In, OutUnit> {
    func: Func,
    in_ph: std::marker::PhantomData<In>,
    out_unit_ph: std::marker::PhantomData<OutUnit>,
}

impl<Func, In, OutUnit> ProbeOutcome<Func, In, OutUnit>
where
    Func: Fn(In) -> OutUnit,
{
    fn new(func: Func) -> Self {
        Self {
            func,
            in_ph: std::marker::PhantomData,
            out_unit_ph: std::marker::PhantomData,
        }
    }
}

impl<Func, In, OutUnit> Outcome for ProbeOutcome<Func, In, OutUnit>
where
    Func: Send + Sync + 'static,
    In: ForeignOutcome<OutUnit>,
    OutUnit: Send + Sync + 'static,
    <<In as ForeignOutcome<OutUnit>>::Quantify as Quantify<OutUnit>>::Output: Send + Sync + 'static,
{
    type Unit = OutUnit;
    type Output = <In::Quantify as Quantify<OutUnit>>::Output;
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

    pub struct ForeignProbeShim<'c, T1, T2, O: Outcome, Func, OutUnit: async_graphql::ContainerType> {
        field: Foreign<T1, T2, O>,
        probe_mech: ProbeOutcome<Func, O::Unit, OutUnit>,
        ctx: &'c ::async_graphql::context::Context<'c>,
    }

    impl<'c, T1, T2, O, Func, OutUnit> ForeignProbeShim<'c, T1, T2, O, Func, OutUnit>
    where
        O: Outcome,
        OutUnit: async_graphql::ContainerType,
    {
        pub fn new(
            field: Foreign<T1, T2, O>,
            probe_mech: ProbeOutcome<Func, O::Unit, OutUnit>,
            ctx: &'c ::async_graphql::context::Context<'c>,
        ) -> Self {
            Self {
                field,
                probe_mech,
                ctx,
            }
        }
    }

    impl<'c, T1, T2, O, Func, OutUnit> ProjectFrom for ForeignProbeShim<'c, T1, T2, O, Func, OutUnit>
    where
        T1: Table,
        T2: Table + Instance,
        O: ForeignOutcome<OutUnit>,
        Func: Send + Sync + 'static,
        <<O as ForeignOutcome<OutUnit>>::Quantify as Quantify<OutUnit>>::Output:
            Send + Sync + 'static,
        OutUnit: async_graphql::ContainerType + Send + Sync + 'static,
    {
        type Table = T1;
        type Outcome = ProbeOutcome<Func, O, OutUnit>;
    }

    impl<'c, T1, T2, O, Func, OutUnit> ProjectAndProbe
        for ForeignProbeShim<'c, T1, T2, O, Func, OutUnit>
    where
        T1: Table,
        T2: Table + Instance,
        O: Outcome<Unit = Node<T2>> + ForeignOutcome<OutUnit>,
        Func: (Fn(<O as Outcome>::Unit) -> OutUnit) + Send + Sync + 'static,
        <<O as ForeignOutcome<OutUnit>>::Quantify as Quantify<OutUnit>>::Output:
            Send + Sync + 'static,
        OutUnit: async_graphql::ContainerType + Send + Sync + 'static,
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
