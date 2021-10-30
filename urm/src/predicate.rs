use crate::build::{BuildPredicate, BuildRange};
use crate::expr::Expr;

pub struct Predicates<J, F, R> {
    pub join: J,
    pub filter: F,
    pub range: R,
}

pub trait IntoPredicates {
    type Join: BuildPredicate;
    type Filter: BuildPredicate;
    type Range: BuildRange;

    fn into_predicates(self) -> Predicates<Self::Join, Self::Filter, Self::Range>;
}

#[derive(Debug)]
pub struct Eq(pub Expr, pub Expr);

impl BuildPredicate for Eq {
    fn build_predicate(&self, _builder: &mut crate::build::QueryBuilder) {}
}

#[derive(Debug)]
pub struct And(pub Vec<Box<dyn BuildPredicate>>);