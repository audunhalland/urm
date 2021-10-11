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
pub mod query;

pub trait Table: Send + Sync + 'static {
    fn name(&self) -> &'static str;
}

pub trait Where<Q>: Table {}

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
impl<T, F> Project<F, <F::Handler as field::Handler>::Output> for Node<T>
where
    T: Table,
    F: field::Field<Owner = T>,
{
    async fn project(self, field: F) -> UrmResult<<F::Handler as field::Handler>::Output> {
        if let NodeState::Setup(setup) = self.state {
            {
                let mut projection = setup.projection().lock();
                <F::Handler as field::Handler>::project(&field, &mut projection);
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
            <F0::Handler as field::Handler>::Output,
            <F1::Handler as field::Handler>::Output,
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
        <F0::Handler as field::Handler>::Output,
        <F1::Handler as field::Handler>::Output,
    )> {
        if let NodeState::Setup(setup) = self.state {
            {
                let mut projection = setup.projection().lock();
                <F0::Handler as field::Handler>::project(&fields.0, &mut projection);
                <F1::Handler as field::Handler>::project(&fields.1, &mut projection);
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

    impl field::FieldBase for Id {
        fn name(&self) -> &'static str {
            "id"
        }

        fn kind(&self) -> field::FieldKind {
            field::FieldKind::Basic
        }
    }

    impl field::Field for Id {
        type Owner = Tab;
        type Handler = field::BasicHandler<String>;
    }

    impl field::FieldBase for Things {
        fn name(&self) -> &'static str {
            "things"
        }

        fn kind(&self) -> field::FieldKind {
            field::FieldKind::Basic
        }
    }

    impl field::Field for Things {
        type Owner = Tab;
        type Handler = field::ForeignHandler<Tab2>;
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
