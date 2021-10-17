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

pub mod agql;
mod engine;
pub mod field;
pub mod prelude;
pub mod query;

mod experiment;

pub trait Table: Send + Sync + 'static {
    fn name(&self) -> &'static str;
}

pub trait Probe {
    fn probe(&self, ctx: &::async_graphql::context::Context<'_>);
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
    Setup,
    Pending,
    Synchronization,
}

pub type UrmResult<T> = Result<T, UrmError>;

/// Node is somethings which has a place in the query tree,
/// which is publicly exposed.
pub struct Node<T: Table> {
    state: NodeState,
    ph: std::marker::PhantomData<T>,
}

impl<T: Table> Node<T> {
    pub fn clone_setup(&self) -> UrmResult<Self> {
        match &self.state {
            NodeState::Setup(setup) => Ok(Self {
                state: NodeState::Setup(setup.fork()),
                ph: std::marker::PhantomData,
            }),
            _ => Err(UrmError::Setup),
        }
    }

    pub fn get_projection(&self) -> Arc<Mutex<engine::Projection>> {
        match &self.state {
            NodeState::Setup(setup) => setup.projection().clone(),
            _ => panic!(),
        }
    }
}

enum NodeState {
    Setup(engine::ProjectionSetup),
    Ready,
}

#[async_trait]
pub trait Project<'a, Inputs: 'a, Outputs> {
    async fn project(self, args: &'a Inputs) -> UrmResult<Outputs>;
}

#[async_trait]
impl<'a, T, F> Project<'a, F, <F::Describe as field::DescribeField>::Output> for Node<T>
where
    T: Table,
    F: field::Field<Owner = T> + field::ProjectAndProbe + 'a,
{
    async fn project(
        self,
        field: &'a F,
    ) -> UrmResult<<F::Describe as field::DescribeField>::Output> {
        let projection = self.get_projection();

        // TODO: parallelize:
        field.project_and_probe(projection.clone()).await;

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
            <F0::Describe as field::DescribeField>::Output,
            <F1::Describe as field::DescribeField>::Output,
        ),
    > for Node<T>
where
    T: Table,
    F0: field::Field<Owner = T> + field::ProjectAndProbe + 'a,
    F1: field::Field<Owner = T> + field::ProjectAndProbe + 'a,
{
    async fn project(
        self,
        fields: &'a (F0, F1),
    ) -> UrmResult<(
        <F0::Describe as field::DescribeField>::Output,
        <F1::Describe as field::DescribeField>::Output,
    )> {
        let projection = self.get_projection();

        // TODO: parallelize:
        fields.0.project_and_probe(projection.clone()).await;
        fields.1.project_and_probe(projection.clone()).await;

        panic!();
    }
}
