use crate::build::{BuildPredicate, BuildRange};

pub trait Filter<P: BuildPredicate> {
    type Output;

    fn filter(self, p: P) -> Self::Output;
}

pub trait Range<R: BuildRange> {
    type Output;

    fn range(self, r: R) -> Self::Output;
}
