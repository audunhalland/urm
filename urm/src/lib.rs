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

pub mod build;
pub mod expr;
pub mod filter;
pub mod postgres;
pub mod predicate;
pub mod prelude;
pub mod probe;
pub mod project;
pub mod quantify;
pub mod query;

mod engine;
mod experiment;
mod never;

pub trait Database: std::fmt::Debug + Sync + Send + Clone + 'static {}

pub trait Table: Send + Sync + 'static {
    type DB: Database;

    fn name(&self) -> &'static str;
}

/// Provide some &'static instance of a type.
pub trait Instance {
    fn instance() -> &'static Self;
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
        self,
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

impl<T, R> filter::Range<R> for Select<T>
where
    T: Table,
    R: build::BuildRange,
{
    type Output = Self;

    fn range(self, range: R) -> Self::Output {
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

/// This function _projects_ a _probe_, and serves two purposes at the same time:
/// 1. figure out the overall structure and anatomy of a database query (i.e. "query builder")
/// 2. deserialize/provide actual values originating in the database back to the caller, after
///    the query has been successfully executed.
///
/// This async function operates in two completely different modes, based on which _phase_ the
/// associated `Probe::node()` is in.
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
    M: ProjectNode<T>,
{
    arg.project_node(probe.node()).await
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

///
/// A projectable 'point' in a database schema.
///
/// A `Node` represents an anchor point from which to spin off new child projections or sub-queries.
///
/// `Node` exists in two different "phases": Probe and Deserialize. However, this is
/// indiscernible at the type level.
///
/// `urm` works by first _probing_ nodes internally, then _deserializing_ externally, using
/// the same asynchronous user-supplied function.
///
pub struct Node<T: Table> {
    phase: Phase<T::DB>,
    table: std::marker::PhantomData<T>,
}

impl<T: Table> Node<T> {
    pub(crate) fn new_probe(probing: engine::Probing<T::DB>) -> Self {
        Self {
            phase: Phase::Probe(probing),
            table: std::marker::PhantomData,
        }
    }

    pub(crate) fn new_deserialize() -> Self {
        Self {
            phase: Phase::Deserialize,
            table: std::marker::PhantomData,
        }
    }
}

enum Phase<DB: Database> {
    Probe(engine::Probing<DB>),
    Deserialize,
}

///
/// High level trait that enables probing - i.e. hierarchical analysis of
/// some 3rd-party tree-like data structure that can translate into a database query.
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

///
/// Project projectable types given a `Node<T>`.
///
#[async_trait]
pub trait ProjectNode<T: Table> {
    type Output;

    async fn project_node(self, node: &Node<T>) -> UrmResult<Self::Output>;
}

#[async_trait]
impl<T, F> ProjectNode<T> for F
where
    T: Table,
    F: project::ProjectFrom<Table = T> + project::ProjectAndProbe<T::DB>,
{
    type Output = <F::Outcome as project::Outcome>::Output;

    async fn project_node(self, node: &Node<T>) -> UrmResult<Self::Output> {
        match &node.phase {
            Phase::Probe(probing) => {
                self.project_and_probe(probing)?;
                never::never().await
            }
            Phase::Deserialize => Err(UrmError::Deserialization),
        }
    }
}

#[async_trait]
impl<T, F0, F1> ProjectNode<T> for (F0, F1)
where
    T: Table,
    F0: project::ProjectFrom<Table = T> + project::ProjectAndProbe<T::DB>,
    F1: project::ProjectFrom<Table = T> + project::ProjectAndProbe<T::DB>,
{
    type Output = (
        <F0::Outcome as project::Outcome>::Output,
        <F1::Outcome as project::Outcome>::Output,
    );

    async fn project_node(self, node: &Node<T>) -> UrmResult<Self::Output> {
        match &node.phase {
            Phase::Probe(probing) => {
                self.0.project_and_probe(probing)?;
                self.1.project_and_probe(probing)?;
                never::never().await
            }
            Phase::Deserialize => Err(UrmError::Deserialization),
        }
    }
}
