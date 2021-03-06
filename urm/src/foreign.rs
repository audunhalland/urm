//!
//! Foreign projection.
//!

use crate::database::Database;
use crate::engine::{Probing, QueryField};
use crate::filter;
use crate::logic::And;
use crate::lower::{BuildRange, Lower, LowerWhere};
use crate::predicate::{IntoPredicates, Predicates};
use crate::project::{LocalId, ProjectAndProbe, ProjectFrom};
use crate::quantify;
use crate::quantify::Quantify;
use crate::ty::{MapTo, ScalarTyped, Type, Typed};
use crate::{Instance, Node, Probe, Table, UrmResult};

pub trait ProjectForeign:
    ProjectFrom
    + Typed<<Self::ForeignTable as Table>::DB>
    + IntoPredicates<<Self::ForeignTable as Table>::DB>
{
    type ForeignTable: Table + Instance;

    ///
    /// Probe this Foreign, effectively mapping the original
    /// type to the probe-able type `P`.
    ///
    #[cfg(feature = "async_graphql")]
    fn probe_with<'c, F, P>(
        self,
        func: F,
        ctx: &'c ::async_graphql::context::Context<'_>,
    ) -> probe_async_graphql::ForeignProbe<'c, Self, F, P>
    where
        Self::Ty: Type<Unit = Node<Self::ForeignTable>> + MapTo<P>,
        F: Fn(<Self::Ty as Type>::Unit) -> P,
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
/// `Ty` is the original outcome of the mapping (having Unit type `Node<T2>` for probing to work).
///
pub struct Foreign<T1, T2, Ty, W, R> {
    source_table: std::marker::PhantomData<T1>,
    foreign_table: std::marker::PhantomData<T2>,
    ty: std::marker::PhantomData<Ty>,
    filter: W,
    range: R,
}

pub fn foreign<T1, T2, Ty, W>(filter: W) -> Foreign<T1, T2, Ty, W, ()>
where
    T1: Table,
    T2: Table,
    Ty: Type,
    W: Lower<T1::DB> + ScalarTyped<T1::DB, bool>,
{
    Foreign {
        source_table: std::marker::PhantomData,
        foreign_table: std::marker::PhantomData,
        ty: std::marker::PhantomData,
        filter,
        range: (),
    }
}

// TODO: Only applicable for OneToMany
impl<T1, T2, Ty, W, W2, R> filter::Filter<T2::DB, W2> for Foreign<T1, T2, Ty, W, R>
where
    T1: Table,
    T2: Table + Instance,
    Ty: Type,
    W: Lower<T2::DB>,
    W2: Lower<T2::DB> + ScalarTyped<T2::DB, bool>,
    R: BuildRange<T2::DB>,
{
    type Output = Foreign<T1, T2, Ty, And<W, W2>, R>;

    fn filter(self, filter: W2) -> Self::Output {
        Self::Output {
            source_table: std::marker::PhantomData,
            foreign_table: std::marker::PhantomData,
            ty: std::marker::PhantomData,
            filter: And(self.filter, filter),
            range: self.range,
        }
    }
}

// TODO: Only applicable for OneToMany
impl<T1, T2, Ty, W, R, R2> filter::Range<T2::DB, R2> for Foreign<T1, T2, Ty, W, R>
where
    T1: Table,
    T2: Table + Instance,
    Ty: Type,
    R: BuildRange<T2::DB>,
    R2: BuildRange<T2::DB>,
{
    type Output = Foreign<T1, T2, Ty, W, R2>;

    fn range(self, range: R2) -> Self::Output {
        Self::Output {
            source_table: std::marker::PhantomData,
            foreign_table: std::marker::PhantomData,
            ty: std::marker::PhantomData,
            filter: self.filter,
            range,
        }
    }
}

impl<DB, T1, T2, Ty, W, R> Typed<DB> for Foreign<T1, T2, Ty, W, R>
where
    DB: Database,
    T1: Table,
    T2: Table,
    Ty: Type,
    W: Lower<T1::DB>,
    R: BuildRange<T2::DB>,
{
    type Ty = Ty;
}

impl<T1, T2, Ty, W, R> ProjectFrom for Foreign<T1, T2, Ty, W, R>
where
    T1: Table,
    T2: Table,
    Ty: Type,
    W: Lower<T1::DB> + ScalarTyped<T1::DB, bool>,
    R: BuildRange<T2::DB>,
{
    type Table = T1;
}

impl<T1, T2, Ty, W, R> IntoPredicates<T2::DB> for Foreign<T1, T2, Ty, W, R>
where
    T1: Table,
    T2: Table<DB = T1::DB>,
    Ty: Type,
    W: Lower<T1::DB> + ScalarTyped<T1::DB, bool>,
    R: BuildRange<T2::DB>,
{
    type Range = R;

    fn into_predicates(self) -> Predicates<T2::DB, Self::Range> {
        Predicates {
            filter: self.filter.lower_where(),
            range: self.range,
        }
    }
}

impl<T1, T2, Ty, W, R> ProjectForeign for Foreign<T1, T2, Ty, W, R>
where
    T1: Table,
    T2: Table<DB = T1::DB> + Instance,
    Ty: Type,
    W: Lower<T1::DB> + ScalarTyped<T1::DB, bool>,
    R: BuildRange<T2::DB>,
{
    type ForeignTable = T2;
}

/// A projection outcome where there will always be exactly one value.
pub struct OneToOne<T2: Table> {
    foreign: std::marker::PhantomData<T2>,
}

impl<T2: Table> Type for OneToOne<T2> {
    type Unit = Node<T2>;
    type Output = Node<T2>;
}

impl<T2: Table, U> MapTo<U> for OneToOne<T2> {
    type Quantify = quantify::AsSelf;
}

/// A projection outcome where there is either one value or nothing.
pub struct OneToOption<T2: Table> {
    foreign: std::marker::PhantomData<T2>,
}

impl<T2: Table> Type for OneToOption<T2> {
    type Unit = Node<T2>;
    type Output = Option<Node<T2>>;
}

impl<T2: Table, U> MapTo<U> for OneToOption<T2> {
    type Quantify = quantify::AsOption;
}

/// A projection outcome where there are potentially many values.
pub struct OneToMany<T2: Table> {
    foreign: std::marker::PhantomData<T2>,
}

impl<T2: Table> Type for OneToMany<T2> {
    type Unit = Node<T2>;
    type Output = Vec<Node<T2>>;
}

impl<T2: Table, U> MapTo<U> for OneToMany<T2> {
    type Quantify = quantify::AsVec;
}

/// Function wrapper to map from some table node unit `In` into probe `Out`,
/// also acting as the Outcome type for this mapping.
pub struct MapToProbe<In, F, Out> {
    func: F,
    node_unit: std::marker::PhantomData<In>,
    probe: std::marker::PhantomData<Out>,
}

impl<In, F, Out> MapToProbe<In, F, Out> {
    fn new(func: F) -> Self {
        Self {
            func,
            node_unit: std::marker::PhantomData,
            probe: std::marker::PhantomData,
        }
    }
}

impl<In, F, Out> Type for MapToProbe<In, F, Out>
where
    In: MapTo<Out>,
    F: Send + Sync + 'static,
    Out: Send + Sync + 'static,
    <<In as MapTo<Out>>::Quantify as Quantify<Out>>::Output: Send + Sync + 'static,
{
    type Unit = Out;
    type Output = <In::Quantify as Quantify<Out>>::Output;
}

#[cfg(feature = "async_graphql")]
pub mod probe_async_graphql {
    use super::*;

    ///
    /// A foreign projection mapped into a probe-able `async_graphql::ContainerType`.
    ///
    pub struct ForeignProbe<'c, In, F, Out>
    where
        In: ProjectForeign,
        Out: async_graphql::ContainerType,
    {
        project_foreign: In,
        map_to_probe:
            MapToProbe<<<In as Typed<<In::ForeignTable as Table>::DB>>::Ty as Type>::Unit, F, Out>,
        ctx: &'c ::async_graphql::context::Context<'c>,
    }

    impl<'c, In, F, P> ForeignProbe<'c, In, F, P>
    where
        In: ProjectForeign,
        P: async_graphql::ContainerType,
    {
        pub(crate) fn new(
            project_foreign: In,
            map_to_probe: MapToProbe<
                <<In as Typed<<In::ForeignTable as Table>::DB>>::Ty as Type>::Unit,
                F,
                P,
            >,
            ctx: &'c ::async_graphql::context::Context<'c>,
        ) -> Self {
            Self {
                project_foreign,
                map_to_probe,
                ctx,
            }
        }
    }

    impl<'c, In, F, Out> Typed<<In::ForeignTable as Table>::DB> for ForeignProbe<'c, In, F, Out>
    where
        In: ProjectForeign,
        In::Ty: MapTo<Out>,
        <<In::Ty as MapTo<Out>>::Quantify as Quantify<Out>>::Output: Send + Sync + 'static,
        F: Send + Sync + 'static,
        Out: async_graphql::ContainerType + Send + Sync + 'static,
    {
        type Ty = MapToProbe<In::Ty, F, Out>;
    }

    impl<'c, In, F, Out> ProjectFrom for ForeignProbe<'c, In, F, Out>
    where
        In: ProjectForeign,
        In::Ty: MapTo<Out>,
        <<In::Ty as MapTo<Out>>::Quantify as Quantify<Out>>::Output: Send + Sync + 'static,
        F: Send + Sync + 'static,
        Out: async_graphql::ContainerType + Send + Sync + 'static,
    {
        type Table = In::Table;
    }

    impl<'c, In, F, Out> ProjectAndProbe<<In::ForeignTable as Table>::DB>
        for ForeignProbe<'c, In, F, Out>
    where
        In: ProjectForeign,
        In::Ty: Type<Unit = Node<In::ForeignTable>> + MapTo<Out>,
        <<In::Ty as MapTo<Out>>::Quantify as Quantify<Out>>::Output: Send + Sync + 'static,
        F: (Fn(<In::Ty as Type>::Unit) -> Out) + Send + Sync + 'static,
        Out: Probe + async_graphql::ContainerType + Send + Sync + 'static,
    {
        fn project_and_probe(
            self,
            probing: &Probing<<In::ForeignTable as Table>::DB>,
        ) -> UrmResult<()> {
            let foreign_table = In::ForeignTable::instance();
            let crate::predicate::Predicates { filter, range } =
                self.project_foreign.into_predicates();

            let sub_select = probing
                .engine()
                .query
                .lock()
                .new_select(foreign_table, filter);

            {
                let mut proj_lock = probing.select().projection.lock();

                proj_lock.insert(
                    // FIXME: This should be "dynamic" in some way,
                    // or use a different type of key when projecting.
                    // perhaps keyed by the predicates..?
                    LocalId(0),
                    QueryField::Foreign {
                        select: sub_select.clone(),
                    },
                );
            }

            let sub_node = Node::<In::ForeignTable>::new_probe(crate::engine::Probing::new(
                probing.engine().clone(),
                sub_select,
            ));

            let container = (self.map_to_probe.func)(sub_node);
            crate::probe::probe_container(&container, self.ctx);

            Ok(())
        }
    }
}
