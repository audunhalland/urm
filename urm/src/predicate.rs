use crate::build::{Build, BuildRange};
use crate::ty::Erased;
use crate::Database;

pub trait Predicate {}

pub struct Predicates<DB: Database, R> {
    pub filter: Option<Box<dyn Build<DB, Ty = Erased>>>,
    pub range: R,
}

pub trait IntoPredicates<DB: Database> {
    type Range: BuildRange<DB>;

    fn into_predicates(self) -> Predicates<DB, Self::Range>;
}
