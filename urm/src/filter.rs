use crate::build::{Build, BuildRange};
use crate::Database;

pub trait Filter<DB: Database, P: Build<DB>> {
    type Output;

    fn filter(self, p: P) -> Self::Output;
}

pub trait Range<DB: Database, R: BuildRange<DB>> {
    type Output;

    fn range(self, r: R) -> Self::Output;
}
