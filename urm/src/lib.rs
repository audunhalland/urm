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

mod engine;
pub mod field;
mod never;
pub mod prelude;
pub mod probe;
pub mod query;

mod experiment;

pub trait Table: Send + Sync + 'static {
    fn name(&self) -> &'static str;
}

/// Provide some &'static instance of a type
pub trait Instance {
    fn instance() -> &'static Self;
}

pub struct Select<T: Table> {
    t: std::marker::PhantomData<T>,
}

impl<T> Select<T>
where
    T: Table + Instance,
{
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

pub fn select<T: Table>() -> Select<T> {
    Select {
        t: std::marker::PhantomData,
    }
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
}

enum NodeKind {
    Probe(engine::Probing),
    Deserialize,
}

///
/// # Project
///
pub async fn project<T, P, M>(probe: &P, arg: &M) -> UrmResult<M::Output>
where
    T: Table,
    P: Probe<Table = T>,
    M: MapProject<T>,
{
    arg.map_project(probe.node()).await
}

pub trait Probe {
    type Table: Table;

    fn node(&self) -> &Node<Self::Table>;
}

#[async_trait]
pub trait MapProject<T: Table> {
    type Output;

    async fn map_project(&self, node: &Node<T>) -> UrmResult<Self::Output>;
}

#[async_trait]
impl<T, F> MapProject<T> for F
where
    T: Table,
    F: field::Field<Table = T> + field::ProjectAndProbe,
{
    type Output = <F::Mechanics as field::FieldMechanics>::Output;

    async fn map_project(&self, node: &Node<T>) -> UrmResult<Self::Output> {
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

    async fn map_project(&self, node: &Node<T>) -> UrmResult<Self::Output> {
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
