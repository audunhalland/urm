use crate::database::Database;
use crate::lower::{Lower, Lowered};
use crate::ty::Typed;
use crate::ty::{Nullable, ScalarTyped};

enum LogicOp {
    And,
    Or,
}

/// Logical conjuction of two clauses.
///
/// To use more than two clauses, just nest it: `And(x, And(y, z))`
pub struct And<L, R>(pub L, pub R);

impl<DB, L, R> Typed<DB> for And<L, R>
where
    DB: Database,
    L: Lower<DB> + ScalarTyped<DB, bool>,
    R: Lower<DB> + ScalarTyped<DB, bool>,
{
    type Ty = Nullable<bool>;
}

impl<DB, L, R> Lower<DB> for And<L, R>
where
    DB: Database,
    L: Lower<DB> + ScalarTyped<DB, bool>,
    R: Lower<DB> + ScalarTyped<DB, bool>,
{
    fn lower(self) -> Option<Lowered<DB>> {
        match (self.0.lower(), self.1.lower()) {
            (Some(Lowered::And(lhs)), Some(Lowered::And(rhs))) => optimize(
                LogicOp::And,
                lhs.into_iter().chain(rhs.into_iter()).map(Option::Some),
            ),
            (Some(Lowered::And(lhs)), rhs) => optimize(
                LogicOp::And,
                lhs.into_iter()
                    .map(Option::Some)
                    .chain(Some(rhs).into_iter()),
            ),
            (lhs, Some(Lowered::And(rhs))) => optimize(
                LogicOp::And,
                Some(lhs)
                    .into_iter()
                    .chain(rhs.into_iter().map(Option::Some)),
            ),
            (lhs, rhs) => optimize(LogicOp::And, vec![lhs, rhs].into_iter()),
        }
    }
}

/// Logical disjunction of two clauses.
///
/// To use more than two clauses, just nest it: `And(x, And(y, z))`
pub struct Or<L, R>(pub L, pub R);

impl<DB, L, R> Typed<DB> for Or<L, R>
where
    DB: Database,
    L: Lower<DB> + ScalarTyped<DB, bool>,
    R: Lower<DB> + ScalarTyped<DB, bool>,
{
    type Ty = Nullable<bool>;
}

impl<DB, L, R> Lower<DB> for Or<L, R>
where
    DB: Database,
    L: Lower<DB> + ScalarTyped<DB, bool>,
    R: Lower<DB> + ScalarTyped<DB, bool>,
{
    fn lower(self) -> Option<Lowered<DB>> {
        match (self.0.lower(), self.1.lower()) {
            (Some(Lowered::Or(lhs)), Some(Lowered::Or(rhs))) => optimize(
                LogicOp::Or,
                lhs.into_iter().chain(rhs.into_iter()).map(Option::Some),
            ),
            (Some(Lowered::Or(lhs)), rhs) => optimize(
                LogicOp::Or,
                lhs.into_iter()
                    .map(Option::Some)
                    .chain(Some(rhs).into_iter()),
            ),
            (lhs, Some(Lowered::Or(rhs))) => optimize(
                LogicOp::Or,
                Some(lhs)
                    .into_iter()
                    .chain(rhs.into_iter().map(Option::Some)),
            ),
            (lhs, rhs) => optimize(LogicOp::Or, vec![lhs, rhs].into_iter()),
        }
    }
}

fn optimize<DB: Database>(
    op: LogicOp,
    clause_iter: impl Iterator<Item = Option<Lowered<DB>>>,
) -> Option<Lowered<DB>> {
    let clauses: Vec<_> = clause_iter.filter_map(|clause| clause).collect();
    match clauses.len() {
        0 => None,
        1 => Some(clauses.into_iter().next().unwrap()),
        _ => Some(match op {
            LogicOp::And => Lowered::And(clauses),
            LogicOp::Or => Lowered::Or(clauses),
        }),
    }
}
