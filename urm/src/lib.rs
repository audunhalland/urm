//!
//! ```text
//! 1.        2.
//!
//!     /D       |/C
//!   B         B|
//!  /  \E     / |\D
//! A         A  |___
//!  \  /F     \  /E|
//!   C         C   |
//!     \G        \F|
//! ```
//!

use async_trait::*;

pub use urm_macros::*;

pub mod expr;
pub mod field;
pub mod filter;
pub mod prelude;
pub mod probe;
pub mod query;

mod engine;
mod experiment;
mod never;

pub trait Table: Send + Sync + 'static {
    fn name(&self) -> &'static str;
}

/// Provide some &'static instance of a type.
pub trait Instance {
    fn instance() -> &'static Self;
}

pub trait Constrain {
    type Table: Table;

    fn filter<F>(self, filter: F) -> Self
    where
        F: filter::Filter<Self::Table>;
}

pub struct Select<T: Table> {
    table: std::marker::PhantomData<T>,
}

impl<T> Select<T>
where
    T: Table + Instance,
{
    /// Perform probing for the select, thus building a suitable query
    /// to send to the database.
    #[cfg(feature = "async_graphql")]
    pub async fn probe_with<F, U>(
        &self,
        func: F,
        ctx: &async_graphql::Context<'_>,
    ) -> UrmResult<Vec<U>>
    where
        F: Fn(Node<T>) -> U,
        U: async_graphql::ContainerType,
    {
        let table = T::instance();
        let (engine, probing) = engine::Engine::new_select(table);
        let node = Node::<T>::new_probe(probing);

        let container = func(node);

        probe::probe_container(&container, ctx);

        let query_engine = engine.query.clone();
        let dbg = format!("{:?}", query_engine.lock());

        Err(UrmError::DebugSelect(dbg))
    }
}

impl<T> Constrain for Select<T>
where
    T: Table,
{
    type Table = T;

    fn filter<F>(self, _filter: F) -> Self
    where
        F: filter::Filter<T>,
    {
        self
    }
}

pub fn select<T>() -> Select<T>
where
    T: Table,
{
    Select {
        table: std::marker::PhantomData,
    }
}

///
/// # Project
///
/// This function _projects_ a _probe_, and serves two purposes at the same time:
/// 1. figure out the overall structure and anatomy of a database query (i.e. "query builder")
/// 2. deserialize/provide actual values originating in the database back to the caller, after
///    the query has been successfully executed.
///
/// This async function operates in two completely different modes, based on which `Node` state
/// is acquired from the Probe.
///
/// If that node is a probing node, the produced future will *never* resolve to a ready value.
/// If that node is a deserialization node, the produced future will try to yield the requested values.
///
/// Therefore, in a probing infrastructure, `project` should always be invoked _as early as possible_
/// at each graph node. In fact, the async infrastructure is treated as _synchronous_ in a probing
/// setup, therefore `project` must be the first future that is awaited at each projection root.
///
pub async fn project<T, P, M>(probe: &P, arg: M) -> UrmResult<M::Output>
where
    T: Table,
    P: Probe<Table = T>,
    M: MapProject<T>,
{
    arg.map_project(probe.node()).await
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum UrmError {
    #[error("Probe error")]
    Probe,

    #[error("Deserialization error")]
    Deserialization,

    #[error("Debug select: {0}")]
    DebugSelect(String),
}

pub type UrmResult<T> = Result<T, UrmError>;

/// Node is somethings which has a place in the query tree,
/// which is publicly exposed.
pub struct Node<T: Table> {
    kind: NodeKind,
    table: std::marker::PhantomData<T>,
}

impl<T: Table> Node<T> {
    pub fn new_probe(probing: engine::Probing) -> Self {
        Self {
            kind: NodeKind::Probe(probing),
            table: std::marker::PhantomData,
        }
    }

    pub fn new_deserialize() -> Self {
        Self {
            kind: NodeKind::Deserialize,
            table: std::marker::PhantomData,
        }
    }
}

enum NodeKind {
    Probe(engine::Probing),
    Deserialize,
}

///
/// # Probe
///
/// The probing procedure involves looking at related database projections
/// as a tree, recursively producing a new tree of queries/subqueries that
/// can later be executed on some database instace.
///
/// This trait is implemented for probe-able types.
/// A `Probe` type will typically represent one "node" in a query tree,
/// and therefore this trait has an associated type `Table`, which is the
/// database table queried at this level in the tree.
///
pub trait Probe {
    type Table: Table;

    fn node(&self) -> &Node<Self::Table>;
}

#[async_trait]
pub trait MapProject<T: Table> {
    type Output;

    async fn map_project(self, node: &Node<T>) -> UrmResult<Self::Output>;
}

#[async_trait]
impl<T, F> MapProject<T> for F
where
    T: Table,
    F: field::Field<Table = T> + field::ProjectAndProbe,
{
    type Output = <F::Mechanics as field::FieldMechanics>::Output;

    async fn map_project(self, node: &Node<T>) -> UrmResult<Self::Output> {
        match &node.kind {
            NodeKind::Probe(probing) => {
                self.project_and_probe(probing)?;
                never::never().await
            }
            NodeKind::Deserialize => Err(UrmError::Deserialization),
        }
    }
}

#[async_trait]
impl<T, F0, F1> MapProject<T> for (F0, F1)
where
    T: Table,
    F0: field::Field<Table = T> + field::ProjectAndProbe,
    F1: field::Field<Table = T> + field::ProjectAndProbe,
{
    type Output = (
        <F0::Mechanics as field::FieldMechanics>::Output,
        <F1::Mechanics as field::FieldMechanics>::Output,
    );

    async fn map_project(self, node: &Node<T>) -> UrmResult<Self::Output> {
        match &node.kind {
            NodeKind::Probe(probing) => {
                self.0.project_and_probe(probing)?;
                self.1.project_and_probe(probing)?;
                never::never().await
            }
            NodeKind::Deserialize => Err(UrmError::Deserialization),
        }
    }
}
