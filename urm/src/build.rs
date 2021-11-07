use crate::builder::QueryBuilder;
use crate::ty::{Erased, Typed};
use crate::Database;

pub trait Build<DB: Database>: Typed<DB> + Send + Sync + 'static {
    fn build(&self, builder: &mut QueryBuilder<DB>);
}

struct ErasedBuild<T>(T);

impl<T, DB> Typed<DB> for ErasedBuild<T>
where
    DB: Database,
{
    type Ty = Erased;
}

impl<T, DB> Build<DB> for ErasedBuild<T>
where
    T: Build<DB> + Send + Sync + 'static,
    DB: Database,
{
    fn build(&self, builder: &mut QueryBuilder<DB>) {
        self.0.build(builder);
    }
}

pub trait IntoErasedBuild<DB>
where
    DB: Database,
{
    fn into_erased_build(self) -> Box<dyn Build<DB, Ty = Erased>>;
}

impl<T, DB> IntoErasedBuild<DB> for T
where
    DB: Database,
    T: Build<DB>,
{
    fn into_erased_build(self) -> Box<dyn Build<DB, Ty = Erased>> {
        Box::new(ErasedBuild(self))
    }
}

pub trait BuildRange<DB: Database>: std::fmt::Debug + Send + Sync + 'static {
    fn build_range(&self, builder: &mut QueryBuilder<DB>);
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
