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

pub struct ProbeSelect<T: Table> {
    t: std::marker::PhantomData<T>,
}

impl<T> ProbeSelect<T>
where
    T: Table + Instance,
{
    #[cfg(feature = "async_graphql")]
    pub async fn map<F, U>(&self, func: F, ctx: &async_graphql::Context<'_>) -> UrmResult<Vec<U>>
    where
        F: Fn(Node<T>) -> U,
        U: async_graphql::ContainerType,
    {
        let table = T::instance();
        let (engine, probing) = engine::Engine::new_select(table);
        let node = Node::<T>::new_probe(probing);

        let container = func(node);

        probe::probe_container_field(&container, ctx);

        let query_engine = engine.query.clone();
        let dbg = format!("{:?}", query_engine);

        Err(UrmError::CannotSelect(dbg))
    }
}

pub fn probe_select<T: Table>() -> ProbeSelect<T> {
    ProbeSelect {
        t: std::marker::PhantomData,
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum UrmError {
    #[error("Probe error")]
    Probe,

    #[error("Cannot select {0}")]
    CannotSelect(String),
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
    FetchResult,
}

#[async_trait]
pub trait Project<'a, Inputs: 'a, Outputs> {
    async fn project(&self, args: &'a Inputs) -> UrmResult<Outputs>;
}

#[async_trait]
impl<'a, T, F> Project<'a, F, <F::Mechanics as field::FieldMechanics>::Output> for Node<T>
where
    T: Table,
    F: field::Field<Table = T> + field::ProjectAndProbe + 'a,
{
    async fn project(
        &self,
        field: &'a F,
    ) -> UrmResult<<F::Mechanics as field::FieldMechanics>::Output> {
        match &self.kind {
            NodeKind::Probe(probing) => {
                field.project_and_probe(probing.engine(), probing.projection())?;
                never::never().await
            }
            NodeKind::FetchResult => {
                panic!();
            }
        }
    }
}

// TODO: macro to generate this impl for N-tuples
#[async_trait]
impl<'a, T, F0, F1>
    Project<
        'a,
        (F0, F1),
        (
            <F0::Mechanics as field::FieldMechanics>::Output,
            <F1::Mechanics as field::FieldMechanics>::Output,
        ),
    > for Node<T>
where
    T: Table,
    F0: field::Field<Table = T> + field::ProjectAndProbe + 'a,
    F1: field::Field<Table = T> + field::ProjectAndProbe + 'a,
{
    async fn project(
        &self,
        fields: &'a (F0, F1),
    ) -> UrmResult<(
        <F0::Mechanics as field::FieldMechanics>::Output,
        <F1::Mechanics as field::FieldMechanics>::Output,
    )> {
        match &self.kind {
            NodeKind::Probe(probing) => {
                fields
                    .0
                    .project_and_probe(probing.engine(), &probing.projection())?;
                fields
                    .1
                    .project_and_probe(probing.engine(), probing.projection())?;
                never::never().await
            }
            NodeKind::FetchResult => {
                panic!();
            }
        }
    }
}
