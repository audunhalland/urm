use super::{Field, FieldMechanics, LocalId, ProjectAndProbe};
use crate::engine::{Probing, QueryField};
use crate::expr;
use crate::{Instance, Node, Table, UrmResult};

pub trait ForeignField: Field {
    type ForeignTable: Table + Instance;

    fn join_predicate(&self, local: expr::TableAlias, foreign: expr::TableAlias)
        -> expr::Predicate;

    fn filter<F>(self, _filter: F) -> Self
    where
        F: crate::filter::Filter<Self::ForeignTable>,
    {
        self
    }

    /// Make a field probe-able by supplying a mapper
    /// function and probing context
    #[cfg(feature = "async_graphql")]
    fn probe_with<'c, T, Func, Out>(
        self,
        func: Func,
        ctx: &'c ::async_graphql::context::Context<'_>,
    ) -> probe_shim::ForeignProbeShim<'c, Self, Func, Self::Mechanics, Out>
    where
        T: Table,
        Self::Mechanics: FieldMechanics<Unit = Node<T>> + ForeignMechanics<Out>,
        Func: Fn(<Self::Mechanics as FieldMechanics>::Unit) -> Out,
        Out: async_graphql::ContainerType,
    {
        probe_shim::ForeignProbeShim::new(self, ProbeMapping::new(func), ctx)
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
    T2: Table,
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

    fn join_predicate(
        &self,
        local_table: expr::TableAlias,
        foreign_table: expr::TableAlias,
    ) -> expr::Predicate {
        self.join_predicate.clone()
    }
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
        F: ForeignField,
        Func,
        InType: FieldMechanics,
        Out: async_graphql::ContainerType,
    > {
        field: F,
        probe_mapping: ProbeMapping<Func, InType::Unit, Out>,
        ctx: &'c ::async_graphql::context::Context<'c>,
    }

    impl<'c, F, Func, InType, Out> ForeignProbeShim<'c, F, Func, InType, Out>
    where
        F: ForeignField,
        InType: FieldMechanics,
        Out: async_graphql::ContainerType,
    {
        pub fn new(
            field: F,
            probe_mapping: ProbeMapping<Func, InType::Unit, Out>,
            ctx: &'c ::async_graphql::context::Context<'c>,
        ) -> Self {
            Self {
                field,
                probe_mapping,
                ctx,
            }
        }
    }

    impl<'c, F, Func, InType, Out> Field for ForeignProbeShim<'c, F, Func, InType, Out>
    where
        F: ForeignField<Mechanics = InType>,
        Func: Send + Sync + 'static,
        InType: ForeignMechanics<Out>,
        <<InType as ForeignMechanics<Out>>::Quantify as Quantify<Out>>::Output:
            Send + Sync + 'static,
        Out: async_graphql::ContainerType + Send + Sync + 'static,
    {
        // This is still the source table, not the target table (T)
        type Table = F::Table;

        type Mechanics = ProbeMapping<Func, F::Mechanics, Out>;
    }

    impl<'c, F, Func, InType, Out> ProjectAndProbe for ForeignProbeShim<'c, F, Func, InType, Out>
    where
        F: ForeignField<Mechanics = InType>,
        Func: (Fn(<InType as FieldMechanics>::Unit) -> Out) + Send + Sync + 'static,
        InType: FieldMechanics<Unit = Node<F::ForeignTable>> + ForeignMechanics<Out>,
        <<InType as ForeignMechanics<Out>>::Quantify as Quantify<Out>>::Output:
            Send + Sync + 'static,
        Out: async_graphql::ContainerType + Send + Sync + 'static,
    {
        fn project_and_probe(&self, probing: &Probing) -> UrmResult<()> {
            let foreign_table = F::ForeignTable::instance();
            let sub_select = probing.engine().query.lock().new_select(foreign_table);

            {
                let mut proj_lock = probing.select().projection.lock();
                proj_lock.insert(
                    // FIXME: This should be "dynamic" in some way,
                    // or use a different type of key when projection.
                    // perhaps keyed by the predicates..?
                    LocalId(0),
                    QueryField::Foreign {
                        select: sub_select.clone(),
                        join_predicate: self
                            .field
                            .join_predicate(probing.select().from.clone(), sub_select.from.clone()),
                    },
                );
            }

            let sub_node = Node::<F::ForeignTable>::new_probe(crate::engine::Probing::new(
                probing.engine().clone(),
                sub_select,
            ));

            let sub_probe = self.probe_mapping.0(sub_node);
            crate::probe::probe_container(&sub_probe, self.ctx);

            Ok(())
        }
    }
}
