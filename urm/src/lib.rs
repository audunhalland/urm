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
use parking_lot::Mutex;
use std::sync::Arc;

pub use urm_macros::*;

mod engine;
pub mod field;
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

impl<T: Table> ProbeSelect<T> {
    pub async fn map<F, U>(&self, _func: F) -> UrmResult<Vec<U>>
    where
        F: Fn(Node<T>) -> U,
    {
        // TODO: The whole algorithm!!
        panic!()
    }
}

pub fn probe_select<T: Table>() -> ProbeSelect<T> {
    ProbeSelect {
        t: std::marker::PhantomData,
    }
}

#[derive(Debug)]
pub enum UrmError {
    Probe,
    Pending,
    Synchronization,
}

pub type UrmResult<T> = Result<T, UrmError>;

/// Node is somethings which has a place in the query tree,
/// which is publicly exposed.
pub struct Node<T: Table> {
    state: NodeState,
    table: std::marker::PhantomData<T>,
}

impl<T: Table> Node<T> {
    pub fn probe(probe_node: engine::ProbeNode) -> Self {
        Self {
            state: NodeState::Probe(probe_node),
            table: std::marker::PhantomData,
        }
    }

    /// Clone this node if it's in Probe state
    pub fn clone_probe(&self) -> UrmResult<Self> {
        match &self.state {
            NodeState::Probe(setup) => Ok(Self {
                state: NodeState::Probe(setup.fork()),
                table: std::marker::PhantomData,
            }),
            _ => Err(UrmError::Probe),
        }
    }

    pub fn get_projection(&self) -> Arc<Mutex<engine::Projection>> {
        match &self.state {
            NodeState::Probe(setup) => setup.projection().clone(),
            _ => panic!(),
        }
    }

    pub fn get_setup(
        &self,
    ) -> (
        Arc<Mutex<engine::QueryEngine>>,
        Arc<Mutex<engine::Projection>>,
    ) {
        match &self.state {
            NodeState::Probe(setup) => (setup.query_engine().clone(), setup.projection().clone()),
            _ => panic!(),
        }
    }
}

enum NodeState {
    Probe(engine::ProbeNode),
    Ready,
}

#[async_trait]
pub trait Project<'a, Inputs: 'a, Outputs> {
    async fn project(self, args: &'a Inputs) -> UrmResult<Outputs>;
}

#[async_trait]
impl<'a, T, F> Project<'a, F, <F::Mechanics as field::FieldMechanics>::Output> for Node<T>
where
    T: Table,
    F: field::Field<Table = T> + field::ProjectAndProbe + 'a,
{
    async fn project(
        self,
        field: &'a F,
    ) -> UrmResult<<F::Mechanics as field::FieldMechanics>::Output> {
        let (query_engine, projection) = self.get_setup();

        // TODO: parallelize:
        field
            .project_and_probe(&query_engine, projection.clone())
            .await?;

        panic!();
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
        self,
        fields: &'a (F0, F1),
    ) -> UrmResult<(
        <F0::Mechanics as field::FieldMechanics>::Output,
        <F1::Mechanics as field::FieldMechanics>::Output,
    )> {
        let (query_engine, projection) = self.get_setup();

        // TODO: parallelize:
        fields
            .0
            .project_and_probe(&query_engine, projection.clone())
            .await?;
        fields
            .1
            .project_and_probe(&query_engine, projection.clone())
            .await?;

        panic!();
    }
}
