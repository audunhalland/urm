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
    fn enter_select(&mut self);
    fn exit_select(&mut self);
}

pub struct PGQueryBuilder;

impl PGQueryBuilder {
    pub fn new() -> Self {
        Self
    }
}

impl QueryBuilder for PGQueryBuilder {
    fn enter_select(&mut self) {}
    fn exit_select(&mut self) {}
}
