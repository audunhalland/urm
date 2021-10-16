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
pub mod prelude;
pub mod query;

mod experiment;

pub trait Table: Send + Sync + 'static {
    fn name(&self) -> &'static str;
}

pub trait DbProxy {
    type Table;
}

#[derive(Debug)]
pub enum UrmError {
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

enum NodeState {
    Setup(engine::ProjectionSetup),
    Ready,
}

#[async_trait]
pub trait Project<Inputs, Outputs> {
    async fn project(self, args: Inputs) -> UrmResult<Outputs>;
}

#[async_trait]
impl<T, F> Project<F, <F::Project as field::ProjectField>::Output> for Node<T>
where
    T: Table,
    F: field::Field<Owner = T>,
{
    async fn project(self, field: F) -> UrmResult<<F::Project as field::ProjectField>::Output> {
        if let NodeState::Setup(setup) = self.state {
            {
                let mut projection = setup.projection().lock();
                <F::Project as field::ProjectField>::project(&field, &mut projection);
            }
            setup.complete().await?;
        }
        panic!()
    }
}

#[async_trait]
impl<T, F0, F1>
    Project<
        (F0, F1),
        (
            <F0::Project as field::ProjectField>::Output,
            <F1::Project as field::ProjectField>::Output,
        ),
    > for Node<T>
where
    T: Table,
    F0: field::Field<Owner = T>,
    F1: field::Field<Owner = T>,
{
    async fn project(
        self,
        fields: (F0, F1),
    ) -> UrmResult<(
        <F0::Project as field::ProjectField>::Output,
        <F1::Project as field::ProjectField>::Output,
    )> {
        if let NodeState::Setup(setup) = self.state {
            {
                let mut projection = setup.projection().lock();
                <F0::Project as field::ProjectField>::project(&fields.0, &mut projection);
                <F1::Project as field::ProjectField>::project(&fields.1, &mut projection);
            }
            setup.complete().await?;
        }
        panic!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct Tab;
    struct Tab2;

    struct Id;
    struct Things;

    impl Table for Tab {
        fn name(&self) -> &'static str {
            "tab"
        }
    }

    impl Table for Tab2 {
        fn name(&self) -> &'static str {
            "tab2"
        }
    }

    impl field::Field for Id {
        type Owner = Tab;
        type Project = field::Scalar<String>;

        fn name() -> &'static str {
            "id"
        }

        fn local_id() -> field::LocalId {
            field::LocalId(0)
        }
    }

    impl field::Field for Things {
        type Owner = Tab;
        type Project = field::ForeignOneToMany<Tab2>;

        fn name() -> &'static str {
            "things"
        }

        fn local_id() -> field::LocalId {
            field::LocalId(1)
        }
    }

    impl Tab {
        fn id() -> Id {
            Id
        }

        fn things() -> Things {
            Things
        }
    }

    #[test]
    fn tup() {
        let (engine, setup) = engine::QueryEngine::new_select(&Tab);
        let node = Node {
            state: NodeState::Setup(setup),
            ph: std::marker::PhantomData,
        };

        let f = node.project((Tab::id(), Tab::things()));
    }
}
