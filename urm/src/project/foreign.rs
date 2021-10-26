use super::{LocalId, Outcome, ProjectAndProbe, ProjectFrom};
use crate::engine::{Probing, QueryField};
use crate::expr;
use crate::{Filter, Instance, Node, Table, UrmResult};

/// 'FlatMap' some outcome into type `U`
/// with desired quantification
pub trait FlatMapOutcome<U>: Outcome {
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

    ///
    /// Probe this foreign, effectively mapping the original
    /// outcome type to the probe-able outcome type `P`.
    ///
    #[cfg(feature = "async_graphql")]
    pub fn probe_with<'c, T, F, P>(
        self,
        func: F,
        ctx: &'c ::async_graphql::context::Context<'_>,
    ) -> probe_async_graphql::ForeignProbe<'c, T1, T2, O, F, P>
    where
        T: Table,
        O: Outcome<Unit = Node<T>> + FlatMapOutcome<P>,
        F: Fn(<O as Outcome>::Unit) -> P,
        P: async_graphql::ContainerType,
    {
        probe_async_graphql::ForeignProbe::new(self, MapToProbe::new(func), ctx)
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

impl<T: Table, U> FlatMapOutcome<U> for OneToOne<T> {
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

impl<T: Table, U> FlatMapOutcome<U> for OneToMany<T> {
    type Quantify = Vector;
}

/// Function wrapper to map from some table node unit `N` into probe `P`,
/// also acting as the Outcome type for this mapping.
pub struct MapToProbe<F, N, P> {
    func: F,
    node_unit: std::marker::PhantomData<N>,
    probe: std::marker::PhantomData<P>,
}

impl<F, N, P> MapToProbe<F, N, P> {
    fn new(func: F) -> Self {
        Self {
            func,
            node_unit: std::marker::PhantomData,
            probe: std::marker::PhantomData,
        }
    }
}

impl<F, N, P> Outcome for MapToProbe<F, N, P>
where
    F: Send + Sync + 'static,
    N: FlatMapOutcome<P>,
    P: Send + Sync + 'static,
    <<N as FlatMapOutcome<P>>::Quantify as Quantify<P>>::Output: Send + Sync + 'static,
{
    type Unit = P;
    type Output = <N::Quantify as Quantify<P>>::Output;
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
pub mod probe_async_graphql {
    use super::*;

    ///
    /// A foreign projection mapped into a probe-able `async_graphql::ContainerType`.
    ///
    /// `T1` is the source table.
    /// `T2` is the target table.
    /// `O` is the original outcome.
    /// `F` is a function that maps to the probe type.
    /// `P` _is_ the probe type, the `ContainerType`.
    ///
    pub struct ForeignProbe<'c, T1, T2, O: Outcome, F, P: async_graphql::ContainerType> {
        foreign: Foreign<T1, T2, O>,
        map_to_probe: MapToProbe<F, O::Unit, P>,
        ctx: &'c ::async_graphql::context::Context<'c>,
    }

    impl<'c, T1, T2, O, F, P> ForeignProbe<'c, T1, T2, O, F, P>
    where
        O: Outcome,
        P: async_graphql::ContainerType,
    {
        pub fn new(
            foreign: Foreign<T1, T2, O>,
            map_to_probe: MapToProbe<F, O::Unit, P>,
            ctx: &'c ::async_graphql::context::Context<'c>,
        ) -> Self {
            Self {
                foreign,
                map_to_probe,
                ctx,
            }
        }
    }

    impl<'c, T1, T2, O, F, P> ProjectFrom for ForeignProbe<'c, T1, T2, O, F, P>
    where
        T1: Table,
        T2: Table + Instance,
        O: FlatMapOutcome<P>,
        F: Send + Sync + 'static,
        <<O as FlatMapOutcome<P>>::Quantify as Quantify<P>>::Output: Send + Sync + 'static,
        P: async_graphql::ContainerType + Send + Sync + 'static,
    {
        type Table = T1;
        type Outcome = MapToProbe<F, O, P>;
    }

    impl<'c, T1, T2, O, F, P> ProjectAndProbe for ForeignProbe<'c, T1, T2, O, F, P>
    where
        T1: Table,
        T2: Table + Instance,
        O: Outcome<Unit = Node<T2>> + FlatMapOutcome<P>,
        F: (Fn(<O as Outcome>::Unit) -> P) + Send + Sync + 'static,
        <<O as FlatMapOutcome<P>>::Quantify as Quantify<P>>::Output: Send + Sync + 'static,
        P: async_graphql::ContainerType + Send + Sync + 'static,
    {
        fn project_and_probe(self, probing: &Probing) -> UrmResult<()> {
            let foreign_table = T2::instance();
            let sub_select = probing
                .engine()
                .query
                .lock()
                .new_select(foreign_table, self.foreign.predicate);

            {
                let mut proj_lock = probing.select().projection.lock();
                let join_predicate = self.foreign.join_predicate;

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

            let container = (self.map_to_probe.func)(sub_node);
            crate::probe::probe_container(&container, self.ctx);

            Ok(())
        }
    }
}
