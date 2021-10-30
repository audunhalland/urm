//!
//! Foreign projection.
//!

use super::{LocalId, Outcome, ProjectAndProbe, ProjectFrom};
use crate::engine::{Probing, QueryField};
use crate::expr;
use crate::quantify;
use crate::quantify::Quantify;
use crate::{Filter, Instance, Node, Table, UrmResult};

/// 'FlatMap' some Outcome into the type `U`
/// having the desired quantification.
pub trait FlatMapOutcome<U>: Outcome {
    type Quantify: Quantify<U>;
}

pub trait ProjectForeign: ProjectFrom {
    type ForeignTable: Table + Instance;

    ///
    /// Probe this Foreign, effectively mapping the original
    /// outcome type to the probe-able outcome type `P`.
    ///
    #[cfg(feature = "async_graphql")]
    fn probe_with<'c, F, P>(
        self,
        func: F,
        ctx: &'c ::async_graphql::context::Context<'_>,
    ) -> probe_async_graphql::ForeignProbe<'c, Self, F, P>
    where
        Self::Outcome: Outcome<Unit = Node<Self::ForeignTable>> + FlatMapOutcome<P>,
        F: Fn(<Self::Outcome as Outcome>::Unit) -> P,
        P: async_graphql::ContainerType,
    {
        let map_to_probe = MapToProbe::new(func);

        probe_async_graphql::ForeignProbe::new(self, map_to_probe, ctx)
    }
}

///
/// A projection using a _foreign key_, leading into a foreign table.
///
/// `T1` is the outer table.
/// `T2` is the inner table.
/// `O` is the original outcome of the mapping (having Unit type `Node<T2>` for probing to work).
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

impl<T1, T2, O> ProjectForeign for Foreign<T1, T2, O>
where
    T1: Table,
    T2: Table + Instance,
    O: Outcome,
{
    type ForeignTable = T2;
}

/// A projection outcome where there will always be exactly one value.
pub struct OneToOne<T2: Table> {
    foreign: std::marker::PhantomData<T2>,
}

impl<T2: Table> Outcome for OneToOne<T2> {
    type Unit = Node<T2>;
    type Output = Node<T2>;
}

impl<T2: Table, U> FlatMapOutcome<U> for OneToOne<T2> {
    type Quantify = quantify::AsSelf;
}

/// A projection outcome where there is either one value or nothing.
pub struct OneToOption<T2: Table> {
    foreign: std::marker::PhantomData<T2>,
}

impl<T2: Table> Outcome for OneToOption<T2> {
    type Unit = Node<T2>;
    type Output = Option<Node<T2>>;
}

impl<T2: Table, U> FlatMapOutcome<U> for OneToOption<T2> {
    type Quantify = quantify::AsOption;
}

/// A projection outcome where there are potentially many values.
pub struct OneToMany<T2: Table> {
    foreign: std::marker::PhantomData<T2>,
}

impl<T2: Table> Outcome for OneToMany<T2> {
    type Unit = Node<T2>;
    type Output = Vec<Node<T2>>;
}

impl<T2: Table, U> FlatMapOutcome<U> for OneToMany<T2> {
    type Quantify = quantify::AsVec;
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
    pub struct ForeignProbe<'c, PF, F, P>
    where
        PF: ProjectForeign,
        P: async_graphql::ContainerType,
    {
        project_foreign: PF,
        map_to_probe: MapToProbe<F, <<PF as ProjectFrom>::Outcome as Outcome>::Unit, P>,
        ctx: &'c ::async_graphql::context::Context<'c>,
    }

    impl<'c, PF, F, P> ForeignProbe<'c, PF, F, P>
    where
        PF: ProjectForeign,
        P: async_graphql::ContainerType,
    {
        pub(crate) fn new(
            project_foreign: PF,
            map_to_probe: MapToProbe<F, <<PF as ProjectFrom>::Outcome as Outcome>::Unit, P>,
            ctx: &'c ::async_graphql::context::Context<'c>,
        ) -> Self {
            Self {
                project_foreign,
                map_to_probe,
                ctx,
            }
        }
    }

    impl<'c, PF, F, P> ProjectFrom for ForeignProbe<'c, PF, F, P>
    where
        PF: ProjectForeign,
        F: Send + Sync + 'static,
        PF::Outcome: FlatMapOutcome<P>,
        <<PF::Outcome as FlatMapOutcome<P>>::Quantify as Quantify<P>>::Output:
            Send + Sync + 'static,
        P: async_graphql::ContainerType + Send + Sync + 'static,
    {
        type Table = PF::Table;
        type Outcome = MapToProbe<F, PF::Outcome, P>;
    }

    impl<'c, PF, F, P> ProjectAndProbe for ForeignProbe<'c, PF, F, P>
    where
        PF: ProjectForeign,
        PF::Outcome: Outcome<Unit = Node<PF::ForeignTable>> + FlatMapOutcome<P>,
        <<PF::Outcome as FlatMapOutcome<P>>::Quantify as Quantify<P>>::Output:
            Send + Sync + 'static,
        F: (Fn(<PF::Outcome as Outcome>::Unit) -> P) + Send + Sync + 'static,
        P: async_graphql::ContainerType + Send + Sync + 'static,
    {
        fn project_and_probe(self, probing: &Probing) -> UrmResult<()> {
            let foreign_table = PF::ForeignTable::instance();
            let sub_select = probing
                .engine()
                .query
                .lock()
                // FIXME: 'filter predicate':
                .new_select(foreign_table, None);

            {
                let mut proj_lock = probing.select().projection.lock();
                // FIXME:
                let join_predicate = crate::expr::Predicate::And(vec![]);

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

            let sub_node = Node::<PF::ForeignTable>::new_probe(crate::engine::Probing::new(
                probing.engine().clone(),
                sub_select,
            ));

            let container = (self.map_to_probe.func)(sub_node);
            crate::probe::probe_container(&container, self.ctx);

            Ok(())
        }
    }
}
