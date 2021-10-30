use crate::{Database, Table};

pub trait BuildPredicate<DB: Database>: std::fmt::Debug + Send + Sync + 'static {
    fn build_predicate(&self, builder: &mut QueryBuilder<DB>);
}

pub trait BuildRange<DB: Database>: std::fmt::Debug + Send + Sync + 'static {
    fn build_range(&self, builder: &mut QueryBuilder<DB>);
}

pub struct QueryBuilder<DB: Database> {
    db: std::marker::PhantomData<DB>,
}

impl<DB: Database> QueryBuilder<DB> {
    pub fn push_str(&mut self, str: &str) {}
}

impl<DB: Database> BuildPredicate<DB> for () {
    fn build_predicate(&self, _builder: &mut QueryBuilder<DB>) {}
}

impl<DB: Database> BuildRange<DB> for () {
    fn build_range(&self, _builder: &mut QueryBuilder<DB>) {}
}

impl<DB: Database> BuildRange<DB> for ::std::ops::Range<usize> {
    fn build_range(&self, _builder: &mut QueryBuilder<DB>) {}
}

impl<DB: Database> BuildRange<DB> for ::std::ops::Range<Option<usize>> {
    fn build_range(&self, _builder: &mut QueryBuilder<DB>) {}
}
