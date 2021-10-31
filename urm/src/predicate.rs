use crate::build::{BuildPredicate, BuildRange, Ctx};
use crate::builder::QueryBuilder;
use crate::expr::Expr;
use crate::Database;

pub trait Predicate {}

pub struct Predicates<J, F, R> {
    pub join: J,
    pub filter: F,
    pub range: R,
}

pub trait IntoPredicates<DB: Database> {
    type Join: BuildPredicate<DB>;
    type Filter: BuildPredicate<DB>;
    type Range: BuildRange<DB>;

    fn into_predicates(self) -> Predicates<Self::Join, Self::Filter, Self::Range>;
}

#[derive(Debug)]
pub struct Eq<DB: Database>(pub Expr<DB>, pub Expr<DB>);

impl<DB: Database> BuildPredicate<DB> for Eq<DB> {
    fn build_predicate(&self, builder: &mut QueryBuilder<DB>, ctx: &Ctx<DB>) {
        self.0.build_expr(builder, ctx);
        builder.push(" = ");
        self.1.build_expr(builder, ctx);
    }
}

#[derive(Debug)]
pub struct And<DB: Database>(pub Vec<Box<dyn BuildPredicate<DB>>>);
