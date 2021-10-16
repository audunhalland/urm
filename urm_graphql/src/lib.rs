//! Graph(QL) experiment

// TODO: Remove
mod regular_graphql;

pub struct Select<T: urm::Table> {
    t: std::marker::PhantomData<T>,
}

impl<T: urm::Table> Select<T> {
    pub async fn map<F, U>(&self, _func: F) -> urm::UrmResult<Vec<U>>
    where
        F: Fn(urm::Node<T>) -> U,
    {
        // TODO: The whole algorithm
        panic!()
    }
}

pub fn select<T: urm::Table>() -> Select<T> {
    Select {
        t: std::marker::PhantomData,
    }
}

pub trait UrmObject {}
