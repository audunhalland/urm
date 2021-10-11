//!
//! Definition of the db-agnostic
//!

pub enum Predicate {
    And(Vec<Predicate>),
    Or(Vec<Predicate>),
    Eq(),
}

pub trait ToQuery {
    fn to_query(&self, builder: &mut dyn QueryBuilder);
}

pub trait QueryBuilder {
    fn push_select(&mut self);
    fn pop_select(&mut self);
}
