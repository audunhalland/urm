use crate::lower::{BuildRange, Lower};
use crate::Database;

pub trait Filter<DB: Database, P: Lower<DB>> {
    type Output;

    fn filter(self, p: P) -> Self::Output;
}

pub trait Range<DB: Database, R: BuildRange<DB>> {
    type Output;

    fn range(self, r: R) -> Self::Output;
}
