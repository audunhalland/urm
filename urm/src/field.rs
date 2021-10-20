use crate::engine::{Probing, QueryField};
use crate::expr;
use crate::{Instance, Node, Table, UrmResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LocalId(pub u16);

///
/// A field having some data type, that can be found in some database.
///
pub trait Field: Sized + Send + Sync {
    /// The table that owns this field
    type Table: Table;

    /// The field 'mechanics', which determines how the field
    /// behaves in the API
    type Mechanics: FieldMechanics;

    fn name(&self) -> &'static str;

    fn local_id() -> LocalId;
}

pub trait ForeignField: Field {
    type ForeignTable: Table + Instance;

    fn join_predicate(&self, local: expr::TableExpr, foreign: expr::TableExpr) -> expr::Predicate;

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

/// Field mechanics
pub trait FieldMechanics: Sized {
    /// Unit of this field type, in case Output is quantified
    type Unit: Send + Sync + 'static;

    /// Final, quantified value of the field (possibly Vec<Self::Unit>).
    type Output: Send + Sync + 'static;
}

/// Something that can be probe-projected directly
pub trait ProjectAndProbe: Field {
    fn project_and_probe(&self, probing: &Probing) -> UrmResult<()>;
}

/// Quantification of some unit value into quantified output
/// for the probing process
pub trait ForeignMechanics<U>: FieldMechanics + Send + Sync + 'static {
    type Quantify: Quantify<U>;
}

///
/// Primitive field type that is just a 'column',
/// not a foreign reference.
///
pub struct Primitive<T> {
    table: std::marker::PhantomData<T>,
}

impl<V> FieldMechanics for Primitive<V>
where
    V: Send + Sync + 'static,
{
    type Unit = V;
    type Output = V;
}

impl<F, V> ProjectAndProbe for F
where
    F: Field<Mechanics = Primitive<V>>,
{
    fn project_and_probe(&self, probing: &Probing) -> UrmResult<()> {
        probing
            .select()
            .projection
            .lock()
            .insert(F::local_id(), QueryField::Primitive);
        Ok(())
    }
}

/// A 'foreign' reference field that points to
/// at most one foreign entity
pub struct ForeignOneToOne<T: Table> {
    foreign: std::marker::PhantomData<T>,
}

impl<T: Table> FieldMechanics for ForeignOneToOne<T> {
    type Unit = Node<T>;
    type Output = Node<T>;
}

impl<T: Table, U> ForeignMechanics<U> for ForeignOneToOne<T> {
    type Quantify = Unit;
}

/// A 'foreign' reference field that points to
/// potentially many foreign entities.
pub struct ForeignOneToMany<T: Table> {
    foreign: std::marker::PhantomData<T>,
}

impl<T: Table> FieldMechanics for ForeignOneToMany<T> {
    type Unit = Node<T>;
    type Output = Vec<Node<T>>;
}

impl<T: Table, U> ForeignMechanics<U> for ForeignOneToMany<T> {
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

        fn name(&self) -> &'static str {
            self.field.name()
        }

        fn local_id() -> LocalId {
            F::local_id()
        }
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
                    F::local_id(),
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
