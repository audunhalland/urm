use crate::database::Database;
use crate::lower::{BuildRange, Lowered};

pub trait Predicate {}

pub struct Predicates<DB: Database, R> {
    pub filter: Option<Lowered<DB>>,
    pub range: R,
}

pub trait IntoPredicates<DB: Database> {
    type Range: BuildRange<DB>;

    fn into_predicates(self) -> Predicates<DB, Self::Range>;
}
