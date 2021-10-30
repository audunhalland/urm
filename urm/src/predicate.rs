use crate::build::{BuildPredicate, BuildRange};
use crate::expr::Expr;
use crate::Database;

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
pub struct Eq<DB: Database>(pub Expr<DB>, pub Expr<DB>);

impl<DB: Database> BuildPredicate for Eq<DB> {
    fn build_predicate(&self, builder: &mut crate::build::QueryBuilder) {
        self.0.build_expr(builder);
        builder.push_str(" = ");
        self.1.build_expr(builder);
    }
}

#[derive(Debug)]
pub struct And(pub Vec<Box<dyn BuildPredicate>>);
